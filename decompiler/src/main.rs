#![allow(dead_code, unused_variables)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::BufWriter;

use memmap::MmapOptions;
use powerpc::{gpr_constants::*, EncodedInstruction};
use work_set::WorkSet;

use crate::fact::basic_block::{BasicBlockFact, BasicBlockFactBuilder};
use crate::fact::basic_block_end::BasicBlockEndFact;
use crate::fact::branch_target::BranchTargetFact;
use crate::fact::parse_error::ParseErrorFact;
use crate::fact::subroutine::SubroutineFact;
use crate::fact::subroutine_call::SubroutineCallFact;
use crate::fact_database::FactDatabase;
use crate::locale::LocaleFormat;
use crate::powerpc_symbolic::{MachineState, MemoryBlock, Write};

mod fact;
mod fact_database;
mod locale;
mod powerpc_symbolic;

fn main() {
    let file = File::open("Super Smash Bros. Melee (v1.02).iso").unwrap();
    let disc_image = unsafe { MmapOptions::new().map(&file) }.unwrap();

    let disc = gamecube_disc::Reader::new(&*disc_image);
    assert_eq!(disc.header().game_code(), "GALE");
    assert_eq!(disc.header().maker_code(), "01");
    assert_eq!(disc.header().disc_id(), 0);
    assert_eq!(disc.header().version(), 2);

    let dol = disc.main_executable();
    analyze(dol, 0x803631e4);
}

fn analyze(dol: dol::Reader, entry_point: u32) {
    let mut db = FactDatabase::new();

    // Mark the entry point.
    db.insert_fact_with(entry_point, || SubroutineFact);

    let mut addrs_to_scan = WorkSet::new();
    addrs_to_scan.insert(entry_point);
    while let Some(addr) = addrs_to_scan.peek().copied() {
        scan_and_close_addrs(dol, addr, &mut db, &mut addrs_to_scan);
    }

    print_annotated_assembly(dol, &db);

    println!();
    println!(
        "scanned {} instructions",
        LocaleFormat(&addrs_to_scan.iter_known().count()),
    );

    print_errors(&db);

    build_basic_blocks(&mut db, entry_point);
    write_graphviz_basic_blocks(dol, &db);

    build_expressions(dol, &mut db, entry_point);
}

/// Scans instructions and records facts until the first diverging branch or closed address is
/// encountered.
///
/// Closes addresses in `addrs_to_scan` as it goes. Inserts branch targets (both local branches and
/// subroutine calls) into `addrs_to_scan`.
fn scan_and_close_addrs(
    dol: dol::Reader,
    mut addr: u32,
    db: &mut FactDatabase,
    addrs_to_scan: &mut WorkSet<u32>,
) {
    loop {
        // Fetch and parse the instruction.
        if !addrs_to_scan.close(addr) {
            break;
        }
        let data = dol.read(addr);
        let instruction = match EncodedInstruction(data).parse(addr) {
            Ok(instruction) => instruction,
            Err(e) => {
                // Parse error. Record the error and abort scanning.
                db.insert_fact_with(addr, || ParseErrorFact::new(e));
                break;
            }
        };

        // Handle branch instructions.
        if let Some(branch_info) = instruction.branch_info() {
            if let Some(target) = branch_info.target {
                // This branch has a static target. It's either a subroutine call or a local branch.

                // Record the branch target.
                db.fact_or_default::<BranchTargetFact>(target)
                    .record_source(addr);
                addrs_to_scan.insert(target);

                if branch_info.link {
                    // This is a subroutine call.
                    db.insert_fact_with(addr, || SubroutineCallFact::new(target));
                    db.insert_fact_with(target, || SubroutineFact);
                } else {
                    // This is a local branch, which marks the end of a basic block and links to one
                    // or two successors.
                    let successor_fact = db.fact_or_default::<BasicBlockEndFact>(addr);
                    successor_fact.record_successor(target);
                    if branch_info.is_conditional() {
                        successor_fact.record_successor(addr + 4);
                    }
                }
            } else if !branch_info.link {
                // This branch has a dynamic target and it's not a subroutine call. Assume it's a
                // return. Mark the end of a basic block with no successors.
                db.fact_or_default::<BasicBlockEndFact>(addr);
            }

            if branch_info.diverges() {
                break;
            }
        }
        addr += 4;
    }
}

fn print_annotated_assembly(dol: dol::Reader, db: &FactDatabase) {
    println!("# annotated assembly");

    for (addr, facts) in db.iter_facts() {
        // Add a space before starting a subroutine.
        if db.get_fact::<SubroutineFact>(addr).is_some() {
            println!();
        }

        // Print displayable facts.
        let mut fact_strings: Vec<_> = facts
            .filter_map(|fact| Some(format!("{}", fact.as_display()?)))
            .collect();
        fact_strings.sort();
        for fact_string in fact_strings {
            println!("            {}", fact_string);
        }

        // Print the assembly listing.
        let data = dol.read(addr);
        print!("0x{:08x}  0x{:08x}  ", addr, data);
        match EncodedInstruction(data).parse(addr) {
            Ok(instruction) => println!("{}", instruction),
            Err(e) => println!("; ERROR: {}", e),
        };
    }
}

fn print_errors(db: &FactDatabase) {
    let mut errors = BTreeSet::new();
    for addr in db.iter_facts_with_type::<ParseErrorFact>() {
        errors.insert(format!(
            "{}",
            db.get_fact::<ParseErrorFact>(addr).unwrap().parse_error(),
        ));
    }

    if !errors.is_empty() {
        println!();
        println!("# errors");
        println!();
        for error in errors {
            println!("{}", error);
        }
    }
}

/// Builds basic blocks and records [`BasicBlockFact`]s.
fn build_basic_blocks(db: &mut FactDatabase, entry_point: u32) {
    println!();
    println!("# decompile pass");
    println!();

    let mut builders_by_addr = BTreeMap::new();

    // Scan all locally connected basic blocks.
    let mut addrs_to_scan = WorkSet::new();
    addrs_to_scan.insert(entry_point);
    while let Some(addr) = addrs_to_scan.pop().copied() {
        let builder = BasicBlockFactBuilder::new(db, addr);
        addrs_to_scan.extend(builder.successors().iter().copied());
        builders_by_addr.insert(addr, builder);
    }
    drop(addrs_to_scan);

    // Populate predecessors for all basic blocks.
    for addr in builders_by_addr.keys().copied().collect::<Vec<u32>>() {
        let targets = builders_by_addr[&addr].successors().to_owned();
        for target in targets {
            builders_by_addr
                .get_mut(&target)
                .unwrap()
                .insert_predecessor(addr);
        }
    }

    // Record basic blocks.
    for (addr, builder) in builders_by_addr {
        db.insert_fact_with(addr, || builder.build());
    }

    println!(
        "located {} basic blocks",
        LocaleFormat(&db.iter_facts_with_type::<BasicBlockFact>().count()),
    );
}

fn write_graphviz_basic_blocks(dol: dol::Reader, db: &FactDatabase) {
    use std::io::Write;

    let mut dot = BufWriter::new(File::create("graph.dot").unwrap());
    writeln!(
        dot,
        r#"digraph G {{
    fontname="sans-serif";
    node [fontname="monospace", style="filled", shape="box"];"#,
    )
    .unwrap();

    for addr in db.iter_facts_with_type::<BasicBlockFact>() {
        let basic_block = db.get_fact::<BasicBlockFact>(addr).unwrap();

        // Emit a graph node.
        write!(
            dot,
            "    \"0x{addr:08x}\" [label=\"[0x{addr:08x}..0x{end_addr:08x}]\\l",
            addr = addr,
            end_addr = basic_block.end_addr(),
        )
        .unwrap();
        for addr in (addr..basic_block.end_addr()).step_by(4) {
            let instruction = EncodedInstruction(dol.read(addr)).parse(addr).unwrap();
            write!(dot, "0x{:08x}  {}\\l", addr, instruction).unwrap();
        }
        write!(dot, "\"];",).unwrap();

        // Emit edges to all successors.
        for target in basic_block.successors().iter().copied() {
            writeln!(dot, "    \"0x{:08x}\" -> \"0x{:08x}\";", addr, target).unwrap();
        }
    }

    writeln!(dot, "}}").unwrap();
}

/// Builds expressions.
fn build_expressions(dol: dol::Reader, db: &mut FactDatabase, entry_point: u32) {
    println!();
    println!("# expression pass");

    let mut machine_state = MachineState::new();
    machine_state.define_memory_block(R1, MemoryBlock::new(-24..8));
    machine_state.define_memory_block(R3, MemoryBlock::new(0..4));

    let mut addrs_to_visit = WorkSet::new();
    addrs_to_visit.insert(entry_point);
    while let Some(addr) = addrs_to_visit.pop().copied() {
        let basic_block = db.get_fact::<BasicBlockFact>(addr).unwrap();
        addrs_to_visit.extend(basic_block.successors().iter().copied());

        println!();
        println!(
            "## basic block 0x{:08x}..0x{:08x}",
            addr,
            basic_block.end_addr()
        );
        println!();

        for addr in (addr..basic_block.end_addr()).step_by(4) {
            let instruction = EncodedInstruction(dol.read(addr)).parse(addr).unwrap();
            let update = machine_state.prepare_update(&instruction);

            let print_prefix = |first: &mut bool| {
                if *first {
                    *first = false;
                    print!("0x{:08x}  {:32}  ", addr, format!("{}", instruction));
                } else {
                    print!("{:46}", "");
                }
            };

            let mut first = true;
            for (register, expr) in &update.registers {
                print_prefix(&mut first);
                println!(
                    "{} <- {}",
                    register,
                    machine_state.ctx().display_expr(*expr),
                );
            }
            for Write { width, addr, data } in &update.writes {
                print_prefix(&mut first);
                println!(
                    "write_{}({}, {})",
                    width,
                    machine_state.ctx().display_expr(*addr),
                    machine_state.ctx().display_expr(*data),
                );
            }
            if first {
                print_prefix(&mut first);
                println!(".");
            }

            machine_state.apply(update);
        }
    }
}
