extern crate cranelift_codegen;
extern crate cranelift_frontend;

use cranelift_codegen::entity::EntityRef;
use cranelift_codegen::ir::types::*;
use cranelift_codegen::ir::{AbiParam, ExternalName, Function, InstBuilder, Signature, ExtFuncData};
use cranelift_codegen::isa::{CallConv, lookup_by_name};
use cranelift_codegen::settings;
use cranelift_codegen::verifier::verify_function;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::immediates::Imm64;
use cranelift_filetests::FunctionRunner;

// function(x) {
// x, y, z : i32
// block0:
//    y = 2;
//    z = x + y;
//    jump block1
// block1:
//    z = z + y;
//    brnz y, block3;
//    jump block2
// block2:
//    z = z - x;
//    return y
// block3:
//    y = y - x
//    jump block1
// }
// ```
// Here is how you build the corresponding Cranelift IR function using `FunctionBuilderContext`:

 fn main() {
     minfunc();
     return;
     let mut sig = Signature::new(CallConv::SystemV);
     sig.returns.push(AbiParam::new(I32));
     sig.params.push(AbiParam::new(I32));
     let mut fn_builder_ctx = FunctionBuilderContext::new();
     let mut func = Function::with_name_signature(ExternalName::user(0, 0), sig);
     {
         let mut builder = FunctionBuilder::new(&mut func, &mut fn_builder_ctx);

         let block0 = builder.create_ebb();
         let block1 = builder.create_ebb();
         let block2 = builder.create_ebb();
         let block3 = builder.create_ebb();
         let x = Variable::new(0);
         let y = Variable::new(1);
         let z = Variable::new(2);
         builder.declare_var(x, I32);
         builder.declare_var(y, I32);
         builder.declare_var(z, I32);
         builder.append_ebb_params_for_function_params(block0);

         builder.switch_to_block(block0);
         builder.seal_block(block0);
         {
             let tmp = builder.ebb_params(block0)[0]; // the first function parameter
             builder.def_var(x, tmp);
         }
         {
             let tmp = builder.ins().iconst(I32, 2);
             builder.def_var(y, tmp);
         }
         {
             let arg1 = builder.use_var(x);
             let arg2 = builder.use_var(y);
             let tmp = builder.ins().iadd(arg1, arg2);
             builder.def_var(z, tmp);
         }
         builder.ins().jump(block1, &[]);

         builder.switch_to_block(block1);
         {
             let arg1 = builder.use_var(y);
             let arg2 = builder.use_var(z);
             let tmp = builder.ins().iadd(arg1, arg2);
             builder.def_var(z, tmp);
         }
         {
             let arg = builder.use_var(y);
             builder.ins().brnz(arg, block3, &[]);
         }
         builder.ins().jump(block2, &[]);

         builder.switch_to_block(block2);
         builder.seal_block(block2);
         {
             let arg1 = builder.use_var(z);
             let arg2 = builder.use_var(x);
             let tmp = builder.ins().isub(arg1, arg2);
             builder.def_var(z, tmp);
         }
         {
             let arg = builder.use_var(y);
             builder.ins().return_(&[arg]);
         }

         builder.switch_to_block(block3);
         builder.seal_block(block3);

         {
             let arg1 = builder.use_var(y);
             let arg2 = builder.use_var(x);
             let tmp = builder.ins().isub(arg1, arg2);
             builder.def_var(y, tmp);
         }
         builder.ins().jump(block1, &[]);
         builder.seal_block(block1);

         builder.finalize();
     }

     let flags = settings::Flags::new(settings::builder());
     let res = verify_function(&func, &flags);
     println!("{}", func.display(None));
     if let Err(errors) = res {
         panic!("{}", errors);
     }
 }

///
/// function (x, y, z: i32, f) -> i32 {
///     if x > y :
///         x = y
///
///     if x > z :
///         x = z
///
///     return f(x)
/// }
fn minfunc() {
    let mut sig = Signature::new(CallConv::SystemV);
    sig.returns.push(AbiParam::new(B1));
    let mut fn_builder_ctx = FunctionBuilderContext::new();
    let mut func = Function::with_name_signature(ExternalName::user(0, 0), sig);
    {
        let mut builder = FunctionBuilder::new(&mut func, &mut fn_builder_ctx);

        let block0 = builder.create_ebb();
        let block1 = builder.create_ebb();
        let block2 = builder.create_ebb();
        let x = Variable::new(0);
        let y = Variable::new(1);
        let z = Variable::new(2);
        builder.declare_var(x, I32);
        builder.declare_var(y, I32);
        builder.declare_var(z, I32);
        builder.append_ebb_params_for_function_params(block0);

        builder.switch_to_block(block0);
        builder.seal_block(block0);
        {
            let xx = builder.ins().iconst(I32, 1);
            let yy = builder.ins().iconst(I32, 2);
            let zz = builder.ins().iconst(I32, 3);
            builder.def_var(x, xx);
            builder.def_var(y, yy);
            builder.def_var(z, zz);
            let left= builder.use_var(x);
            let right = builder.use_var(y);
            builder.ins().br_icmp(IntCC::UnsignedLessThanOrEqual, left,right, block1,&[]);
            builder.def_var(x, right);
            builder.ins().jump(block1, &[]);
        }
        builder.switch_to_block(block1);
        builder.seal_block(block1);
        {
            let left= builder.use_var(x);
            let right = builder.use_var(z);
            builder.ins().br_icmp(IntCC::UnsignedLessThanOrEqual, left,right, block2,&[]);
            builder.def_var(x, right);
            builder.ins().jump(block2, &[]);
        }

        builder.switch_to_block(block2);
        builder.seal_block(block2);
        let temp= builder.use_var(x);
        let mut sig = Signature::new(CallConv::SystemV);
        sig.params.push(AbiParam::new(I32));
        sig.returns.push(AbiParam::new(I32));
        let sig = builder.import_signature(sig);
        let f = builder.ins().iconst(I64, userfunc as usize as i64);
        let inst = builder.ins().call_indirect(sig, f, &[temp] );
        let result = builder.inst_results(inst)[0];
        let b = builder.ins().icmp_imm(IntCC::UnsignedGreaterThan, result, 1);
        builder.ins().return_(&[b]);
        builder.finalize();
    }

    let flags = settings::Flags::new(settings::builder());
    let res = verify_function(&func, &flags);
    println!("{}", func.display(None));
    if let Err(errors) = res {
        panic!("{}", errors);
    }

    let isa_builder = lookup_by_name("x86_64").unwrap();
    let flag_builder = settings::builder();
    let isa = isa_builder.finish(settings::Flags::new(flag_builder));

    FunctionRunner::new(func, isa).run();
}

extern "sysv64" fn userfunc(val: i32) -> i32 {
    println!("call func val:{}", val);
    val
}