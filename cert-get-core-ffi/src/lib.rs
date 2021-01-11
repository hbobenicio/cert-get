use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

// TODO improve error handling (sent back to the user somehow...)
// TODO use a Struct as the input argument
#[no_mangle]
pub extern "C" fn download_certs(
    address: *const c_char,
    output_dir: *const c_char,
    insecure: c_int,
    generate_jks: c_int,
) {
    // Shadow unsafe parameters to safe types straight away
    let address = unsafe { CStr::from_ptr(address) };
    let output_dir = unsafe { CStr::from_ptr(output_dir) };
    let insecure: bool = if insecure == 1 { true } else { false };
    let generate_jks: bool = if generate_jks == 1 { true } else { false };

    let address_str = match address.to_str() {
        Ok(value) => value,
        Err(err) => {
            eprintln!(
                "error: download_certs: url is not a valid utf-8 string: {}",
                err
            );
            return;
        }
    };

    let output_dir_str = match output_dir.to_str() {
        Ok(value) => value,
        Err(err) => {
            eprintln!(
                "error: output_dir: url is not a valid utf-8 string: {}",
                err
            );
            return;
        }
    };

    let download_params = cert_get_core::DownloadParams {
        address: String::from(address_str),
        output_dir: String::from(output_dir_str),
        insecure,
        generate_jks,
    };

    if let Err(err) = cert_get_core::download_certs(&download_params) {
        eprintln!("download_certs failed: {}", err);
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
