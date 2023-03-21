use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;

use serde_json::Value;
use slog::Logger;

use crate::certificate::{Certificate, CertificateError, CertificateLoader};
use crate::settings::Settings;

pub fn make_ca_certificate_loader(settings: Arc<Settings>, logger: Logger) -> CertificateLoader {
    let ca_certificate_file = settings.ca_certificate_file.clone();

    Box::new(move || {
        debug!(logger, "Loading CA certificate");
        if let Some(path) = ca_certificate_file.as_ref() {
            let mut file = File::open(path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            Ok(vec![Certificate::from_pem(&contents)?])
        } else {
            Err(CertificateError::NoFileSpecified("CA certificate".to_owned()))
        }
    })
}

pub fn make_server_certificate_loader(settings: Arc<Settings>, logger: Logger) -> CertificateLoader {
    let server_certificates = settings.server_certificates.clone();
    let private_key_files = settings.private_key_files.clone();
    let certificate_chain_files = settings.certificate_chain_files.clone();

    Box::new(move || {
        debug!(logger, "Loading server certificates");

        if !server_certificates.is_empty() {
            let mut certs = Vec::new();
            for cert in server_certificates.iter() {
                let key_path = cert
                    .get("private_key_file")
                    .and_then(|v| v.as_str())
                    .map(PathBuf::from)
                    .or_else(|| {
                        private_key_files
                            .get(0)
                            .map(|v| PathBuf::from(v.clone()))
                    })
                    .ok_or_else(|| CertificateError::NoFileSpecified("private key".to_owned()))?;

                let chain_path = cert
                    .get("certificate_chain_file")
                    .and_then(|v| v.as_str())
                    .map(PathBuf::from)
                    .or_else(|| {
                        certificate_chain_files
                            .get(0)
                            .map(|v| PathBuf::from(v.clone()))
                    })
                    .ok_or_else(|| CertificateError::NoFileSpecified("certificate chain".to_owned()))?;

                let mut key_file = File::open(&key_path)?;
                let mut key_contents = String::new();
                key_file.read_to_string(&mut key_contents)?;

                let mut chain_file = File::open(&chain_path)?;
                let mut chain_contents = String::new();
                chain_file.read_to_string(&mut chain_contents)?;

                certs.push(Certificate::new(
                    key_contents.trim(),
                    chain_contents.trim(),
                )?);
            }
            Ok(certs)
        } else {
            Err(CertificateError::NoFileSpecified("server certificates".to_owned()))
        }
    })
}

fn make_load_dh_param_handler(settings: &nmos::settings, gate: &mut slog::Logger) -> impl FnMut() -> utility::string_t {
    let dh_param_file = nmos::experimental::fields::dh_param_file(settings);

    move || {
        slog::info!(gate, "Load DH parameters");

        if dh_param_file.is_empty() {
            slog::warning!(gate, "Missing DH parameters file");
        } else {
            let mut dh_file = File::open(PathBuf::from(dh_param_file)).expect("Failed to open DH parameters file");
            let mut dh_param = String::new();
            dh_file.read_to_string(&mut dh_param).expect("Failed to read DH parameters file");
            return dh_param;
        }
        return utility::string_t::new();
    }
}