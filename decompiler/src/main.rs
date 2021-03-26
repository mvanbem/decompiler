use std::fs::File;

use memmap::MmapOptions;
use powerpc::{DecodedInstruction, EncodedInstruction};

fn main() {
    let file = File::open("Super Smash Bros. Melee (v1.02).iso").unwrap();
    let disc_image = unsafe { MmapOptions::new().map(&file) }.unwrap();

    let disc = gamecube_disc::Reader::new(&*disc_image);
    assert_eq!(disc.header().game_code(), "GALE");
    assert_eq!(disc.header().maker_code(), "01");
    assert_eq!(disc.header().disc_id(), 0);
    assert_eq!(disc.header().version(), 2);

    let dol = disc.main_executable();
    analyze_function(dol, 0x803631e4);
}

fn analyze_function(dol: dol::Reader<'_>, mut address: u32) {
    loop {
        let data = dol.read(address);
        let instruction = EncodedInstruction(data).parse(address).unwrap();
        println!("0x{:08x}  0x{:08x}  {}", address, data, instruction);

        if let DecodedInstruction::Bclr { .. } = instruction {
            break;
        }
        address += 4;
    }
}
