use std::{
    ffi::{c_char, c_int, CStr, CString},
    ptr::null_mut,
    sync::{LazyLock, Mutex},
};
use zsh_sys::{
    builtin, execstring, features, featuresarray, handlefeatures, setfeatureenables, HandlerFunc,
    Options,
};

extern "C" fn lastcmd(_name: *mut c_char, args: *mut *mut c_char, _: Options, _: c_int) -> c_int {
    let command = unsafe {
        let mut hist = zsh_sys::hist_ring;
        hist = (*hist).up;
        if !hist.is_null() {
            CStr::from_ptr((*hist).node.nam).to_string_lossy()
        } else {
            return 0;
        }
    };
    let command = unsafe { CStr::from_ptr(*args as _) }
        .to_str()
        .unwrap()
        .replace("!!", &command);
    unsafe {
        execstring(
            CString::new(command).unwrap().as_ptr() as _,
            1,
            0,
            null_mut(),
        );
    }
    0
}

fn new_builtin(
    name: &'static CStr,
    handlerfunc: HandlerFunc,
    minargs: c_int,
    maxargs: c_int,
) -> builtin {
    builtin {
        node: zsh_sys::hashnode {
            next: null_mut(),
            nam: name.as_ptr() as _,
            flags: 0,
        },
        handlerfunc,
        minargs,
        maxargs,
        funcid: 0,
        optstr: null_mut(),
        defopts: null_mut(),
    }
}

struct Module {
    bintab: Vec<builtin>,
    features: features,
}

impl Module {
    fn new() -> Self {
        Module {
            bintab: Vec::new(),
            features: unsafe { std::mem::MaybeUninit::zeroed().assume_init() },
        }
    }
}

unsafe impl Send for Module {}
unsafe impl Sync for Module {}

static MODULE: LazyLock<Mutex<Module>> = LazyLock::new(|| {
    let mut module = Module::new();
    module
        .bintab
        .push(new_builtin(c"lastcmd", Some(lastcmd), 1, 1));
    module.features.bn_list = module.bintab.as_mut_ptr();
    module.features.bn_size = module.bintab.len() as _;
    Mutex::new(module)
});

#[unsafe(no_mangle)]
extern "C" fn setup_(_m: zsh_sys::Module) -> c_int {
    0
}

#[unsafe(no_mangle)]
extern "C" fn features_(m: zsh_sys::Module, features: *mut *mut *mut c_char) -> c_int {
    let mut module = MODULE.lock().unwrap();
    unsafe {
        *features = featuresarray(m, &mut module.features);
    }
    0
}

#[unsafe(no_mangle)]
extern "C" fn enables_(m: zsh_sys::Module, enables: *mut *mut c_int) -> c_int {
    let mut module = MODULE.lock().unwrap();

    unsafe { handlefeatures(m, &mut module.features, enables) }
}

#[unsafe(no_mangle)]
extern "C" fn boot_(_m: zsh_sys::Module) -> c_int {
    0
}

#[unsafe(no_mangle)]
extern "C" fn cleanup_(m: zsh_sys::Module) -> c_int {
    let mut module = MODULE.lock().unwrap();
    unsafe { setfeatureenables(m, &mut module.features, null_mut()) }
}

#[unsafe(no_mangle)]
extern "C" fn finish_(_m: zsh_sys::Module) -> c_int {
    0
}
