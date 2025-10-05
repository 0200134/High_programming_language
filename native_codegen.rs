use crate::ir_generator::IRModule;
use std::fs::File;
use std::io::Write;
use std::process::Command;

pub fn generate_native_binary(ir: &IRModule, asm_path: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let mut asm = String::from("global main\nsection .text\nmain:\n");

    #[cfg(not(target_os = "windows"))]
    let mut asm = String::from("section .text\n global _start\n_start:\n");

    for instr in &ir.instructions {
        match instr.opcode.as_str() {
            "let" => {
                asm.push_str(&format!("  ; let {} = {}\n", instr.operands[0], instr.operands[1]));
            }
            "return" => {
                #[cfg(target_os = "windows")]
                asm.push_str("  mov eax, 0\n  ret\n");

                #[cfg(not(target_os = "windows"))]
                asm.push_str("  mov rax, 60\n  xor rdi, rdi\n  syscall\n");
            }
            _ => {
                asm.push_str("  nop\n");
            }
        }
    }

    let mut file = File::create(asm_path).map_err(|e| e.to_string())?;
    file.write_all(asm.as_bytes()).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn assemble_and_link(asm_path: &str, output_path: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let obj_path = "compiled.obj";

        let nasm_status = Command::new("nasm")
            .args(&["-f", "win64", asm_path, "-o", obj_path])
            .status()
            .map_err(|e| format!("NASM 실행 실패: {}", e))?;

        if !nasm_status.success() {
            return Err("NASM 어셈블 실패".into());
        }

        let gcc_status = Command::new("gcc")
            .args(&[obj_path, "-o", output_path])
            .status()
            .map_err(|e| format!("GCC 링커 실패: {}", e))?;

        if !gcc_status.success() {
            return Err("GCC 링커 실패".into());
        }

        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        let obj_path = "compiled.o";

        let nasm_status = Command::new("nasm")
            .args(&["-f", "elf64", asm_path, "-o", obj_path])
            .status()
            .map_err(|e| format!("NASM 실행 실패: {}", e))?;

        if !nasm_status.success() {
            return Err("NASM 어셈블 실패".into());
        }

        let ld_status = Command::new("ld")
            .args(&[obj_path, "-o", output_path])
            .status()
            .map_err(|e| format!("LD 링커 실패: {}", e))?;

        if !ld_status.success() {
            return Err("LD 링커 실패".into());
        }

        Command::new("chmod")
            .args(&["+x", output_path])
            .status()
            .map_err(|e| format!("실행 권한 부여 실패: {}", e))?;

        Ok(())
    }
}
