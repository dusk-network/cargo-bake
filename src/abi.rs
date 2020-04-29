use std::fs::File;
use std::io::prelude::*;
use std::io::Read;
use std::path::Path;

pub(crate) fn generate(manifest_path: &str) {
  let mut bytes: Vec<u8> = vec![];
  let mut opcode: u8 = 1;

  let out_dir = std::env::current_dir().unwrap();
  let lib = Path::new(manifest_path)
    .parent()
    .unwrap()
    .join("src")
    .join("lib.rs");
  println!("Manifest: {}", manifest_path);
  println!("Lib: {:?}", lib);
  let mut src = String::new();
  let mut file = File::open(&lib).expect("Unable to open file");
  file.read_to_string(&mut src).expect("Unable to read file");

  let syntax = syn::parse_file(&src).expect("Unable to parse file");

  let item = syntax.items.iter().find(|&x| {
    if let syn::Item::Mod(_) = x {
      true
    } else {
      false
    }
  });

  let item = item.unwrap();
  match item {
    syn::Item::Mod(item_mod) => {
      // Incorrectly assume the first module is always the contract's one
      let (_, mod_items) = if let Some(content) = &item_mod.content {
        content
      } else {
        panic!("Contracts cannot be empty.")
      };

      for mod_item in mod_items {
        match mod_item {
          syn::Item::Fn(item_fn) => {
            let vis = &item_fn.vis;
            if let syn::Visibility::Public(_) = vis {
              let name = item_fn.sig.ident.to_string();
              println!("CONTRACT METHOD: {}", name);
              bytes.push(opcode);
              opcode += 1;
              bytes.push(name.len() as u8);
              bytes.extend_from_slice(name.as_bytes());
              // result.push(method(item_fn.clone()).into());
            }
          }
          _ => {}
        }
      }
      // println!("{:#?}", item_mod);
    }
    _ => {}
  }

  if !bytes.is_empty() {
    let path = out_dir.join("transfer.abi");
    let mut file = File::create(path).unwrap();
    file.write_all(&bytes).unwrap();
    println!("Wrote {} bytes", &bytes.len());
  }
}
