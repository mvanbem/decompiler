use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::BufWriter;

use memmap::MmapOptions;
use powerpc::gpr_constants::*;
use powerpc::EncodedInstruction;
use powerpc_symbolic::Variable;
use symbolic::{Expr, ExprRef, NumberedVariable};
use work_set::WorkSet;

use crate::fact::basic_block::{BasicBlockFact, BasicBlockFactBuilder};
use crate::fact::basic_block_end::BasicBlockEndFact;
use crate::fact::branch_target::BranchTargetFact;
use crate::fact::parse_error::ParseErrorFact;
use crate::fact::subroutine::SubroutineFact;
use crate::fact::subroutine_call::SubroutineCallFact;
use crate::fact_database::FactDatabase;
use crate::iter_singleton::IteratorExt;
use crate::locale::LocaleFormat;
use crate::powerpc_symbolic::{Context, MachineState};

mod fact;
mod fact_database;
mod iter_singleton;
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

    for section in dol.iter_sections() {
        println!(
            "section: offset = 0x{:08x}, load_addr = 0x{:08x}, size = 0x{:08x}",
            section.offset, section.load_address, section.size,
        );
    }

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
        write!(dot, "\"];").unwrap();

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
    println!("# first expression pass (local symbolic execution)");

    let basic_block_addrs = {
        let mut basic_block_addrs = Vec::new();
        let mut addrs_to_visit = WorkSet::new();
        addrs_to_visit.insert(entry_point);
        while let Some(basic_block_addr) = addrs_to_visit.pop().copied() {
            basic_block_addrs.push(basic_block_addr);
            addrs_to_visit.extend(
                db.get_fact::<BasicBlockFact>(basic_block_addr)
                    .unwrap()
                    .successors()
                    .iter()
                    .copied(),
            );
        }
        basic_block_addrs.sort_unstable();
        basic_block_addrs
    };

    let mut ctx = Context::new();

    for basic_block_addr in basic_block_addrs.iter().copied() {
        let basic_block = db.get_fact_mut::<BasicBlockFact>(basic_block_addr).unwrap();

        println!();
        println!(
            "## basic block 0x{:08x}..0x{:08x}",
            basic_block_addr,
            basic_block.end_addr()
        );
        println!();

        // Set up a bunch of machine state manually.
        //
        // This will eventually need to be populated automatically from function facts in line with
        // the PowerPC 32-bit C ABI.
        let mut machine_state = MachineState::new(&mut ctx, basic_block_addr);

        // let r1 = machine_state
        //     .ctx_mut()
        //     .variable_expr(Variable::RegisterEntering {
        //         basic_block_addr,
        //         register: R1.into(),
        //     });
        // let r3 = machine_state
        //     .ctx_mut()
        //     .variable_expr(Variable::RegisterEntering {
        //         basic_block_addr,
        //         register: R3.into(),
        //     });
        // let r30 = machine_state
        //     .ctx_mut()
        //     .variable_expr(Variable::RegisterEntering {
        //         basic_block_addr,
        //         register: R30.into(),
        //     });
        // let r31 = machine_state
        //     .ctx_mut()
        //     .variable_expr(Variable::RegisterEntering {
        //         basic_block_addr,
        //         register: R31.into(),
        //     });

        // let stack_memory = machine_state.allocate_memory_block();
        // machine_state.set_memory_pointer(r1, stack_memory);

        // let param0_memory = machine_state.allocate_memory_block();
        // machine_state.set_memory_pointer(r3, param0_memory);
        // let param0_field0 = machine_state
        //     .ctx_mut()
        //     .variable_expr(Variable::EscapeHatch("param0.field0".to_string()));
        // machine_state.write_memory_base_offset(r3, 0, param0_field0);

        // // NOTE: r30 holds the value of r3-on-entry for most of the function. This is wrong, but map it
        // // here until we can do a better job.
        // let param0_memory_dupe = machine_state.allocate_memory_block();
        // machine_state.set_memory_pointer(r30, param0_memory_dupe);
        // // TODO: name this variable!
        // machine_state.write_memory_base_offset(r30, 0, param0_field0);

        // // No idea what this is yet.
        // let r31_memory = machine_state.allocate_memory_block();
        // machine_state.set_memory_pointer(r31, r31_memory);
        // let r31_word0 = machine_state
        //     .ctx_mut()
        //     .variable_expr(Variable::EscapeHatch("r31.word0".to_string()));
        // machine_state.write_memory_base_offset(r31, 0, r31_word0);

        for addr in (basic_block_addr..basic_block.end_addr()).step_by(4) {
            let instruction = EncodedInstruction(dol.read(addr)).parse(addr).unwrap();
            let update = machine_state.prepare_update(addr, &instruction);

            // Don't print anything for calls. It's always the same verbose thing.
            if let Some(branch_info) = instruction.branch_info() {
                if branch_info.link {
                    machine_state.apply(update);
                    continue;
                }
            }

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
            for write in &update.writes {
                basic_block.record_write(*write);
                print_prefix(&mut first);
                println!(
                    "write_{}({}, {})",
                    write.width,
                    machine_state.ctx().display_expr(write.addr),
                    machine_state.ctx().display_expr(write.data),
                );
            }
            if first {
                print_prefix(&mut first);
                println!(".");
            }

            machine_state.apply(update);
        }

        // Record register state on leaving.
        let garbage = machine_state.ctx_mut().variable_expr(Variable::Garbage);
        let registers_leaving: Vec<_> = machine_state.iter_registers().collect();
        let mut print_registers = Vec::new();
        for (register, assignment) in registers_leaving {
            let variable = machine_state
                .ctx_mut()
                .variable_expr(Variable::RegisterLeaving {
                    basic_block_addr,
                    register,
                });
            machine_state
                .ctx_mut()
                .assign_variable(variable, assignment);

            // Don't bother printing garbage (though it's always recorded).
            if assignment != garbage {
                print_registers.push((register, assignment));
            }
        }
        if !print_registers.is_empty() {
            println!();
            println!("registers on leaving basic block:");
            println!();
            print_registers.sort_unstable_by_key(|(register, _)| *register);
            for (register, expr) in print_registers {
                println!("    {} = {}", register, ctx.display_expr(expr));
            }
        }
    }

    println!();
    println!("# second expression pass (work backward from writes & returns)");
    println!();

    // Seed the work set with the variables directly referenced from any memory write or return
    // value expressions.
    let mut exprs_to_visit = WorkSet::new();
    let stack_base = ctx.variable_expr(Variable::RegisterEntering {
        basic_block_addr: entry_point,
        register: R1.into(),
    });
    for basic_block_addr in basic_block_addrs.iter().copied() {
        let basic_block = db.get_fact::<BasicBlockFact>(basic_block_addr).unwrap();
        for write in basic_block.writes() {
            if let Some((base, offset)) = extract_base_offset(&mut ctx, write.addr) {
                if base == stack_base && (offset == 4 || offset < 0) {
                    // This is a write to this function's stack frame. Don't treat it as a root. If
                    // it's referenced elsewhere, it will be found in the tracing phase.
                    println!(
                        "  * not rooting stack write at {} {} {}",
                        ctx.display_expr(base),
                        if offset < 0 { "-" } else { "+" },
                        offset.abs(),
                    );
                    continue;
                }
            }
            println!(
                "  * rooting write_{}({}, {})",
                write.width,
                ctx.display_expr(write.addr),
                ctx.display_expr(write.data),
            );
            exprs_to_visit.insert(write.addr);
            exprs_to_visit.insert(write.data);
        }
        if basic_block.successors().is_empty() {
            let variable = ctx.variable_expr(Variable::RegisterLeaving {
                basic_block_addr,
                register: R3.into(),
            });
            println!("  * rooting return value {}", ctx.display_expr(variable));
            exprs_to_visit.insert(variable);
        }
    }

    // Trace variables and synthesize predecessors.
    while let Some(expr) = exprs_to_visit.pop().copied() {
        // Retrieve this expression as a variable. If it's not a variable, insert its leaves into
        // the work set in its place.
        let variable = match ctx.get_expr(expr) {
            Expr::Variable(variable) => variable,
            _ => {
                for leaf in ctx.get_expr_leaves(expr) {
                    exprs_to_visit.insert(leaf);
                }
                continue;
            }
        };

        // Trace assignments if available. (the main use case for this is memory writes, which
        // generate an assigned variable for indirection)
        if let Some(assignment) = ctx.get_variable_assignment(expr) {
            exprs_to_visit.insert(assignment);
            continue;
        }

        match variable {
            // No implicit predecessor for anonymous temporaries.
            NumberedVariable::Numbered(_) => (),

            NumberedVariable::Named(name) => match name {
                Variable::Garbage => {}

                // Unassigned variables for a register on entering a basic block flow in from
                // predecessor basic blocks.
                Variable::RegisterEntering {
                    basic_block_addr,
                    register,
                } => {
                    // Release the shared borrow on `ctx` above.
                    let basic_block_addr = *basic_block_addr;
                    let register = *register;

                    let basic_block = db.get_fact::<BasicBlockFact>(basic_block_addr).unwrap();
                    if !basic_block.predecessors().is_empty() {
                        // Construct a phi expression referring to the basic block's predecessors.
                        let mut params = Vec::new();
                        for predecessor in basic_block.predecessors().iter().copied() {
                            params.push(ctx.variable_expr(Variable::RegisterLeaving {
                                basic_block_addr: predecessor,
                                register,
                            }));
                        }
                        let assignment = ctx.phi_expr(params);
                        println!(
                            "  * generated assignment: {} := {}",
                            ctx.display_expr(expr),
                            ctx.display_expr(assignment),
                        );
                        ctx.assign_variable(expr, assignment);
                        exprs_to_visit.insert(assignment);
                    }
                }

                // Unassigned variables for a register on leaving a basic block are just the values
                // on entering that basic block. Any changes would have been noted in the first
                // pass.
                Variable::RegisterLeaving {
                    basic_block_addr,
                    register,
                } => {
                    // Release the shared borrow on `ctx` above.
                    let basic_block_addr = *basic_block_addr;
                    let register = *register;

                    let assignment = ctx.variable_expr(Variable::RegisterEntering {
                        basic_block_addr,
                        register,
                    });
                    println!(
                        "  * generated assignment: {} := {}",
                        ctx.display_expr(expr),
                        ctx.display_expr(assignment),
                    );
                    ctx.assign_variable(expr, assignment);
                    exprs_to_visit.insert(assignment);
                }

                // Refers to the return value from a function call.
                Variable::Return { .. } => {
                    // TODO: Emit a C function call.
                }
            },
        }
    }

    println!();
    println!("third expression pass (idk)");

    for basic_block_addr in basic_block_addrs.iter().copied() {
        let basic_block = db.get_fact::<BasicBlockFact>(basic_block_addr).unwrap();

        println!();
        println!(
            "## basic block 0x{:08x}..0x{:08x}",
            basic_block_addr,
            basic_block.end_addr()
        );
        println!();

        for write in basic_block.writes() {
            let resolved_addr = ctx.map_leaves(write.addr, &resolve_variables);
            let resolved_data = ctx.map_leaves(write.data, &resolve_variables);

            println!(
                "write_{}({}, {})",
                write.width,
                ctx.display_expr(resolved_addr),
                ctx.display_expr(resolved_data),
            );
        }

        if basic_block.successors().is_empty() {
            let expr = ctx.variable_expr(Variable::RegisterLeaving {
                basic_block_addr,
                register: R3.into(),
            });
            let resolved_expr = ctx.map_leaves(expr, &resolve_variables);
            println!("return <- {}", ctx.display_expr(resolved_expr));
        }
    }
}

fn resolve_variables(
    ctx: &mut symbolic::Context<NumberedVariable<Variable>>,
    mut expr: ExprRef,
) -> ExprRef {
    loop {
        if let Expr::Variable(_) = ctx.get_expr(expr) {
            if let Some(assignment) = ctx.get_variable_assignment(expr) {
                if expr != assignment {
                    expr = assignment;
                    continue;
                }
            }
        }
        return expr;
    }
}

fn extract_base_offset(ctx: &mut Context, addr: ExprRef) -> Option<(ExprRef, i32)> {
    match ctx.get_expr(addr) {
        // A literal is interpreted as an offset from a special absolute base.
        Expr::Literal(literal) => {
            let literal = *literal as i32;
            Some((ctx.literal_expr(0), literal))
        }

        // A variable is interpreted as a zero-offset reference from its own base.
        Expr::Variable(_) => Some((addr, 0)),

        Expr::Add(exprs) => {
            // There have to be precisely two expressions added together.
            if exprs.len() != 2 {
                return None;
            }

            // One of them has to be a variable.
            let register = exprs
                .iter()
                .copied()
                .filter(|expr| ctx.is_variable(*expr))
                .singleton()?;

            // And one of them has to be a literal.
            let literal = exprs
                .iter()
                .copied()
                .filter_map(|expr| {
                    if let Expr::Literal(literal) = ctx.get_expr(expr) {
                        return Some(*literal);
                    }
                    None
                })
                .singleton()?;

            Some((register, literal as i32))
        }

        // Anything else has no base-offset interpretation.
        _ => None,
    }
}
