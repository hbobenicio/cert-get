use std::ffi::CStr;
use std::os::raw::c_char;

// TODO improve error handling (sent back to the user somehow...)
#[no_mangle]
pub extern "C" fn download_certs(url: *const c_char, output_dir: *const c_char) {
    let url = unsafe { CStr::from_ptr(url) };
    let output_dir = unsafe { CStr::from_ptr(output_dir) };

    let url_str = match url.to_str() {
        Ok(value) => value,
        Err(err) => {
            eprintln!("error: download_certs: url is not a valid utf-8 string: {}", err);
            return;
        }
    };

    let output_dir_str = match output_dir.to_str() {
        Ok(value) => value,
        Err(err) => {
            eprintln!("error: output_dir: url is not a valid utf-8 string: {}", err);
            return;
        }
    };

    if let Err(err) = cert_get_core::download_certs(url_str, output_dir_str) {
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
