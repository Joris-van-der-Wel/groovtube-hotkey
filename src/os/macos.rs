use core_foundation::base::{TCFType};
use core_foundation::string::{CFString};
use core_foundation::dictionary::{CFDictionary};
use core_foundation::boolean::{CFBoolean};

mod bindings {
    use core_foundation::string::CFStringRef;
    use core_foundation::dictionary::CFDictionaryRef;

    extern "C" {
        pub static kAXTrustedCheckOptionPrompt: CFStringRef;
        pub fn AXIsProcessTrusted() -> bool;
        pub fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> bool;
    }
}


pub fn check_accessibility_access(prompt: bool) -> bool {
    unsafe {
        match prompt {
            true => {
                let options: CFDictionary<CFString, CFBoolean> = CFDictionary::from_CFType_pairs(&[
                    (CFString::wrap_under_get_rule(bindings::kAXTrustedCheckOptionPrompt), CFBoolean::true_value()),
                ]);

                bindings::AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef())
            },
            false => {
                bindings::AXIsProcessTrusted()
            },
        }

    }
}
