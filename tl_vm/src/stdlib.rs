use std::{fs::File, io::Read, sync::Arc};

use linked_hash_map::LinkedHashMap;
use tl_core::Module;
use tl_util::Rf;

use crate::{
    const_value::{ConstValue, ConstValueKind, Type},
    scope::{Scope, ScopeValue},
};

pub fn std_module() -> Arc<Module> {
    let file_path = "test_files/std.xl";
    let mut file = File::open(file_path).unwrap();

    let mut input = String::new();
    file.read_to_string(&mut input).unwrap();

    let (module, errors) = Module::parse_str(&input, "std");
    for error in errors {
        println!("{error}")
    }

    Arc::new(module)
}

pub fn fill_module(module_rf: Rf<Scope>) {
    let mut module = module_rf.borrow_mut();

    // module.insert("print", ScopeValue::)

    let io = module.insert(module_rf.clone(), "io".to_string(), ScopeValue::Module(Arc::new(Module::empty("io"))), 0);

    fill_io(&io);
}

pub fn fill_io(module_rf: &Rf<Scope>) {
    // let mut module = module_rf.borrow_mut();

    create_func(
        &module_rf,
        "print",
        [("data".to_string(), Type::String)].into_iter(),
        [].into_iter(),
        Arc::new(|params| {
            if let Some(data) = params.get("data") {
                if let Some(data) = data.resolve_ref() {
                    let ScopeValue::ConstValue(cv) = &data.borrow().value else {
                        return LinkedHashMap::new()
                    };
                    println!("{}", cv)
                } else {
                    println!("{}", data)
                }
            }
            LinkedHashMap::new()
        }),
    );
}

fn create_func<P: Iterator<Item = (String, Type)>, R: Iterator<Item = (String, Type)>>(
    module: &Rf<Scope>,
    name: &str,
    p: P,
    r: R,
    func: Arc<
        dyn Fn(&LinkedHashMap<String, ConstValue>) -> LinkedHashMap<String, ConstValue>
            + Sync
            + Send,
    >,
) -> Rf<Scope> {
    let mut mo = module.borrow_mut();

    let sym = mo.insert(module.clone(), name.to_string(), ScopeValue::Root, 0);

    let cv = ScopeValue::ConstValue(ConstValue {
        kind: ConstValueKind::NativeFunction {
            rf: sym,
            callback: func,
        },
        ty: Type::Function {
            parameters: LinkedHashMap::from_iter(p),
            return_parameters: LinkedHashMap::from_iter(r),
        },
    });

    mo.update(name, cv).unwrap()
}
