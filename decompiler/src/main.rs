use std::any::{Any, TypeId};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::{self, Debug, Display, Formatter};
use std::fs::File;
use std::iter::{empty, once, FromIterator};

use memmap::MmapOptions;
use powerpc::{ConditionBehavior, CtrBehavior, DecodedInstruction, EncodedInstruction, ParseError};

use crate::locale::LocaleFormat;

mod locale;

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

fn analyze(dol: dol::Reader<'_>, entry_point: u32) {
    let mut db = FactDatabase::new();

    // Mark the entry point.
    db.fact_or_default::<BranchTargetFact>(entry_point)
        .record_source(0);
    db.fact_or_default::<SubroutineFact>(entry_point);

    // NOTE: This overall loop structure is n^2 in the number of subroutines discovered. This might
    //       have to be revisited with an out-of-band open subroutine set if it becomes a meaningful
    //       component to performance.
    loop {
        let mut any_progress = false;

        let next_scan_addr = db
            .iter_facts_with_type::<SubroutineFact>()
            // .filter(|addr| *addr == 0x803631e4)
            .filter(|addr| db.get_fact::<ScannedFact>(*addr).is_none())
            .next();

        if let Some(addr) = next_scan_addr {
            scan(dol, addr, &mut db);
            any_progress = true;
        }

        if !any_progress {
            break;
        }
    }

    dump_annotated_assembly(&db);
    dump_counters(&db);
    dump_errors(&db);
}

fn scan(dol: dol::Reader, mut addr: u32, db: &mut FactDatabase) {
    loop {
        // Fetch and parse the instruction.
        let data = dol.read(addr);
        let instruction = match EncodedInstruction(data).parse(addr) {
            Ok(instruction) => instruction,
            Err(e) => {
                // Parse error. Record the error and abort scanning.

                db.new_fact_with(addr, || ScannedFact {
                    data,
                    assembly: e.to_string(),
                });
                db.new_fact_with(addr, || ParseErrorFact(e));
                break;
            }
        };

        // Record the scan.
        db.new_fact_with(addr, || ScannedFact {
            data,
            assembly: format!("{}", instruction),
        });

        match instruction {
            DecodedInstruction::B { link, target, .. } => {
                // Record the branch target.
                db.fact_or_default::<BranchTargetFact>(target)
                    .record_source(addr);

                if link {
                    // The branch target is a subroutine entry point.
                    db.fact_or_default::<SubroutineFact>(target);
                }
            }
            DecodedInstruction::Bc {
                condition,
                ctr,
                link,
                target,
                ..
            } => {
                // Record the branch target.
                db.fact_or_default::<BranchTargetFact>(target)
                    .record_source(addr);

                if link {
                    // The branch target is a subroutine entry point.
                    db.fact_or_default::<SubroutineFact>(target);
                }

                if !link && condition == ConditionBehavior::BranchAlways && ctr == CtrBehavior::None
                {
                    // The following instruction is never reached.
                    break;
                }
            }
            DecodedInstruction::Bclr {
                condition,
                ctr,
                link,
                ..
            } => {
                // The branch target is not statically known.
                // TODO: Try a bit harder at this.

                if !link && condition == ConditionBehavior::BranchAlways && ctr == CtrBehavior::None
                {
                    // The following instruction is never reached.
                    break;
                }
            }
            _ => (),
        }
        addr += 4;
    }
}

fn dump_counters(db: &FactDatabase) {
    println!();
    println!(
        "scanned {} instructions",
        LocaleFormat(&db.iter_facts_with_type::<ScannedFact>().count()),
    );
}

fn dump_annotated_assembly(db: &FactDatabase) {
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
        if let Some(fact) = db.get_fact::<ScannedFact>(addr) {
            println!("0x{:08x}  0x{:08x}  {}", addr, fact.data, fact.assembly);
        } else {
            println!("0x{:08x}  (not scanned)", addr);
        }
    }
}

fn dump_errors(db: &FactDatabase) {
    let mut errors = BTreeSet::new();
    for addr in db.iter_facts_with_type::<ParseErrorFact>() {
        errors.insert(format!(
            "{}",
            db.get_fact::<ParseErrorFact>(addr).unwrap().0
        ));
    }

    if !errors.is_empty() {
        println!();
        println!("*** ERRORS ***");
        for error in errors {
            println!("{}", error);
        }
    }
}

/// Facts may only be inserted, though they may carry mutable state.
struct FactDatabase {
    facts_by_addr: BTreeMap<u32, HashMap<TypeId, Box<dyn Fact>>>,
    facts_by_type_id: HashMap<TypeId, BTreeSet<u32>>,
}

impl FactDatabase {
    fn new() -> FactDatabase {
        FactDatabase {
            facts_by_addr: BTreeMap::new(),
            facts_by_type_id: HashMap::new(),
        }
    }

    fn get_fact<T: Fact>(&self, addr: u32) -> Option<&T> {
        Some(
            self.facts_by_addr
                .get(&addr)?
                .get(&TypeId::of::<T>())?
                .as_any()
                .downcast_ref()
                .unwrap(),
        )
    }

    fn fact_or_default<T: DefaultFact>(&mut self, addr: u32) -> &mut T {
        self.facts_by_type_id
            .entry(TypeId::of::<T>())
            .or_default()
            .insert(addr);
        self.facts_by_addr
            .entry(addr)
            .or_default()
            .entry(TypeId::of::<T>())
            .or_insert_with(|| T::default())
            .as_any_mut()
            .downcast_mut()
            .unwrap()
    }

    fn new_fact_with<T, F>(&mut self, addr: u32, f: F)
    where
        T: ConstructibleFact,
        F: FnOnce() -> T,
    {
        self.facts_by_type_id
            .entry(TypeId::of::<T>())
            .or_default()
            .insert(addr);
        self.facts_by_addr
            .entry(addr)
            .or_default()
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(f()));
    }

    fn iter_facts(
        &self,
    ) -> impl Iterator<Item = (u32, impl Clone + Iterator<Item = &dyn Fact> + '_)> + '_ {
        self.facts_by_addr.iter().map(|(addr, facts)| {
            let facts = facts.values().map(|boxed_fact| &**boxed_fact);
            (*addr, facts)
        })
    }

    fn iter_facts_with_type<T: Fact>(&self) -> Box<dyn Iterator<Item = u32> + '_> {
        match self.facts_by_type_id.get(&TypeId::of::<T>()) {
            Some(addrs) => Box::new(addrs.iter().copied()),
            None => Box::new(empty()),
        }
    }
}

trait Fact: Any + 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn as_display(&self) -> Option<&dyn Display> {
        None
    }
}

trait DefaultFact: Fact {
    fn default() -> Box<Self>
    where
        Self: Sized;
}

trait ConstructibleFact: Fact {
    type Args;

    fn new(args: Self::Args) -> Box<Self>
    where
        Self: Sized;
}

// This address is the target of one or more branch instructions.
#[derive(Default, Debug)]
struct BranchTargetFact {
    sources: BTreeSet<u32>,
}

impl BranchTargetFact {
    pub fn record_source(&mut self, source: u32) {
        self.sources.insert(source);
    }
}

impl Fact for BranchTargetFact {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_display(&self) -> Option<&dyn Display> {
        Some(self)
    }
}

impl DefaultFact for BranchTargetFact {
    fn default() -> Box<Self> {
        Box::new(Default::default())
    }
}

impl Display for BranchTargetFact {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "#[branch_target(sources = [")?;
        let mut first = true;
        for source in self.sources.iter().copied() {
            if first {
                first = false;
            } else {
                write!(f, ", ")?;
            }
            write!(f, "0x{:08x}", source)?;
        }
        write!(f, "])]")
    }
}

impl From<u32> for BranchTargetFact {
    fn from(source: u32) -> Self {
        Self {
            sources: once(source).collect(),
        }
    }
}

impl FromIterator<u32> for BranchTargetFact {
    fn from_iter<T: IntoIterator<Item = u32>>(iter: T) -> Self {
        Self {
            sources: iter.into_iter().collect(),
        }
    }
}

// This address is called as a subroutine. It might be a good candidate for a C function.
#[derive(Default, Debug)]
struct SubroutineFact;

impl Fact for SubroutineFact {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_display(&self) -> Option<&dyn Display> {
        Some(self)
    }
}

impl DefaultFact for SubroutineFact {
    fn default() -> Box<Self> {
        Box::new(Default::default())
    }
}

impl Display for SubroutineFact {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "#[subroutine]")
    }
}

#[derive(Default, Debug)]
struct ScannedFact {
    data: u32,
    assembly: String,
}

impl Fact for ScannedFact {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl ConstructibleFact for ScannedFact {
    type Args = (u32, String);

    fn new((data, assembly): (u32, String)) -> Box<Self> {
        Box::new(ScannedFact { data, assembly })
    }
}

#[derive(Debug)]
struct ParseErrorFact(ParseError);

impl Fact for ParseErrorFact {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl ConstructibleFact for ParseErrorFact {
    type Args = ParseError;

    fn new(err: ParseError) -> Box<Self> {
        Box::new(Self(err))
    }
}
