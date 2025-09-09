// cargo run --example splits --target aarch64-apple-darwin
// cargo run --example splits --target x86_64-apple-darwin

extern crate asr;
extern crate silksong_autosplit_wasm;
#[cfg(not(target_os = "unknown"))]
extern crate serde_json;
extern crate std;

#[cfg(not(target_os = "unknown"))]
use silksong_autosplit_wasm::splits::Split;
#[cfg(not(target_os = "unknown"))]
use serde_json::json;
#[cfg(not(target_os = "unknown"))]
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};
#[cfg(not(target_os = "unknown"))]
use ugly_widget::radio_button::{RadioButtonOption, RadioButtonOptions};

fn main() -> std::io::Result<()> {
    #[cfg(not(target_os = "unknown"))]
    {
        let splits_json = Path::new(file!()).parent().unwrap().join("splits.json");

        let j = serde_json::Value::Array(
            Split::radio_button_options()
                .iter()
                .map(|o| {
                    let RadioButtonOption {
                        key,
                        description,
                        tooltip,
                        ..
                    } = o;
                    json!({ "key": key, "description": description, "tooltip": tooltip })
                })
                .collect(),
        );

        let file = File::create(splits_json)?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, &j)?;
        writeln!(&mut writer)?;
        writer.flush()?;
    }
    
    Ok(())
}
