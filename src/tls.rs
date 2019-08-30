use rustls::{
    AllowAnyAnonymousOrAuthenticatedClient, AllowAnyAuthenticatedClient, KeyLogFile, NoClientAuth,
    RootCertStore, ServerConfig,
};
// use webpki::{TLSServerTrustAnchors, TrustAnchor};
use webpki_roots::TLS_SERVER_ROOTS;
// use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
// use actix_server::ssl::{OpensslAcceptor, RustlsAcceptor};
// use actix_connect::ssl::RustlsConnector;

// use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use super::Opt;
use std::{
    fs::File,
    io::{
        BufReader,
        // prelude::*,
        // Write,
        Read,
    },
    sync::Arc,
    // net,
    // collections::HashMap
};

// fn find_suite(name: &str) -> Option<&'static rustls::SupportedCipherSuite> {
//   for suite in &rustls::ALL_CIPHERSUITES {
//     let sname = format!("{:?}", suite.suite).to_lowercase();

//     if sname == name.to_string().to_lowercase() {
//       return Some(suite);
//     }
//   }

//   None
// }

// fn lookup_suites(suites: &[String]) -> Vec<&'static rustls::SupportedCipherSuite> {
//   let mut out = Vec::new();

//   for csname in suites {
//     let scs = find_suite(csname);
//     match scs {
//       Some(s) => out.push(s),
//       None => panic!("cannot look up ciphersuite '{}'", csname),
//     }
//   }

//   out
// }

/// Make a vector of protocol versions named in `versions`
// fn lookup_versions(versions: &[String]) -> Vec<rustls::ProtocolVersion> {
//   let mut out = Vec::new();

//   for vname in versions {
//     let version = match vname.as_ref() {
//       "1.2" => rustls::ProtocolVersion::TLSv1_2,
//       "1.3" => rustls::ProtocolVersion::TLSv1_3,
//       _ => panic!("cannot look up version '{}', valid are '1.2' and '1.3'", vname),
//     };
//     out.push(version);
//   }

//   out
// }

fn load_certs(filename: &str) -> Vec<rustls::Certificate> {
    let certfile = File::open(filename).expect("cannot open certificate file");
    let mut reader = BufReader::new(certfile);
    rustls::internal::pemfile::certs(&mut reader).unwrap()
}

fn load_private_key(filename: &str) -> rustls::PrivateKey {
    let rsa_keys = {
        let keyfile = File::open(filename).expect("cannot open private key file");
        let mut reader = BufReader::new(keyfile);
        rustls::internal::pemfile::rsa_private_keys(&mut reader)
            .expect("file contains invalid rsa private key")
    };

    let pkcs8_keys = {
        let keyfile = File::open(filename).expect("cannot open private key file");
        let mut reader = BufReader::new(keyfile);
        rustls::internal::pemfile::pkcs8_private_keys(&mut reader)
            .expect("file contains invalid pkcs8 private key (encrypted keys not supported)")
    };

    // prefer to load pkcs8 keys
    if !pkcs8_keys.is_empty() {
        pkcs8_keys[0].clone()
    } else {
        assert!(!rsa_keys.is_empty());
        rsa_keys[0].clone()
    }
}

fn load_ocsp(filename: &str) -> Vec<u8> {
    let mut ret = Vec::new();

    File::open(filename)
        .expect("cannot open ocsp file")
        .read_to_end(&mut ret)
        .unwrap();

    ret
}

pub fn config(opt: &Opt) -> ServerConfig {
    //// OpenSSL example
    // let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    // builder
    //     .set_private_key_file("./examples/key.pem", SslFiletype::PEM)
    //     .unwrap();
    // builder
    //     .set_certificate_chain_file("./examples/cert.pem")
    //     .unwrap();
    // let acceptor = builder.build();

    //// Rustls example
    let client_auth = if let Some(flag_auth) = &opt.flag_auth {
        if !flag_auth.is_empty() {
            let roots = load_certs(&flag_auth);
            let mut client_auth_roots = RootCertStore::empty();
            for root in roots {
                client_auth_roots.add(&root).unwrap();
            }
            if opt.flag_require_auth {
                AllowAnyAuthenticatedClient::new(client_auth_roots)
            } else {
                AllowAnyAnonymousOrAuthenticatedClient::new(client_auth_roots)
            }
        } else {
            NoClientAuth::new()
        }
    } else {
        let mut client_auth_roots = RootCertStore::empty();
        client_auth_roots.add_server_trust_anchors(&TLS_SERVER_ROOTS);
        // println!("set to use mozilla root certs");
        AllowAnyAnonymousOrAuthenticatedClient::new(client_auth_roots)
        // NoClientAuth::new()
    };
    let mut cfg = ServerConfig::new(client_auth);
    // let mut cfg = ServerConfig::new(NoClientAuth::new());

    cfg.key_log = Arc::new(KeyLogFile::new());

    let certs = load_certs(&opt.flag_certs);
    let privkey = load_private_key(&opt.flag_key);
    let ocsp = if opt.flag_ocsp.is_empty() {
        Vec::new()
    } else {
        load_ocsp(&opt.flag_ocsp)
    };
    cfg.set_single_cert_with_ocsp_and_sct(certs, privkey, ocsp, vec![])
        .expect("bad certificates or private keys");

    cfg.set_protocols(
        &opt.flag_proto
            .iter()
            .map(|proto| proto.as_bytes().to_vec())
            .collect::<Vec<_>>()[..],
    );
    cfg
}

// pub fn acceptor(opt : &Opt) -> Result<SslAcceptor, openssl::error::ErrorStack> {
//   let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())?;
//   builder.set_private_key_file(&opt.flag_key, SslFiletype::PEM)?;
//   builder.set_certificate_chain_file(&opt.flag_certs)?;
//   Ok(builder.build())
// }
