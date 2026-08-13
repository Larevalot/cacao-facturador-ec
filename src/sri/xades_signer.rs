//! Firmador electrónico XAdES-BES para comprobantes XML del SRI Ecuador.
//! Utiliza el motor nativo Java (javax.xml.crypto.dsig) garantizando 100% de compatibilidad con la JVM del SRI.

use base64::Engine;
use chrono::Local;
use openssl::hash::MessageDigest;
use openssl::pkcs12::Pkcs12;
use openssl::pkey::PKey;
use openssl::provider::Provider;
use openssl::sign::Signer;
use openssl::x509::X509;
use sha2::{Digest, Sha256};
use std::fs;
use std::process::Command;
use uuid::Uuid;

pub fn firmar_xml(
    p12_bytes: &[u8],
    password: &str,
    xml_content: &str,
) -> Result<String, String> {
    // 1. Intentar firmar con Java (javax.xml.crypto.dsig) para 100% de compatibilidad SRI
    match firmar_xml_java(p12_bytes, password, xml_content) {
        Ok(signed_xml) => Ok(signed_xml),
        Err(e) => {
            eprintln!("[FIRMANDOR] Java signer falló ({}), usando fallback Rust...", e);
            firmar_xml_rust(p12_bytes, password, xml_content)
        }
    }
}

fn firmar_xml_java(
    p12_bytes: &[u8],
    password: &str,
    xml_content: &str,
) -> Result<String, String> {
    // Crear archivos temporales para p12 y xml
    let temp_dir = std::env::temp_dir();
    let p12_path = temp_dir.join(format!("cert_{}.p12", Uuid::new_v4()));
    let xml_path = temp_dir.join(format!("doc_{}.xml", Uuid::new_v4()));

    fs::write(&p12_path, p12_bytes)
        .map_err(|e| format!("Error guardando P12 temporal: {}", e))?;
    fs::write(&xml_path, xml_content.as_bytes())
        .map_err(|e| format!("Error guardando XML temporal: {}", e))?;

    // Ruta de las clases compiladas de Java
    let class_dir = "/home/larevalo/cacaoapps/cacaofacturador-ec/target/classes";

    // Asegurar que SriSigner.class esté compilado
    if !std::path::Path::new(&format!("{}/sri/SriSigner.class", class_dir)).exists() {
        let _ = fs::create_dir_all(class_dir);
        let compile_output = Command::new("javac")
            .arg("-d")
            .arg(class_dir)
            .arg("/home/larevalo/cacaoapps/cacaofacturador-ec/src/sri/SriSigner.java")
            .output();

        if let Err(e) = compile_output {
            let _ = fs::remove_file(&p12_path);
            let _ = fs::remove_file(&xml_path);
            return Err(format!("Error compilando SriSigner.java: {}", e));
        }
    }

    // Ejecutar SriSigner
    let output = Command::new("java")
        .arg("-cp")
        .arg(class_dir)
        .arg("sri.SriSigner")
        .arg(&p12_path)
        .arg(password)
        .arg(&xml_path)
        .output();

    // Limpiar archivos temporales
    let _ = fs::remove_file(&p12_path);
    let _ = fs::remove_file(&xml_path);

    let output = output.map_err(|e| format!("Error ejecutando Java SriSigner: {}", e))?;

    if output.status.success() {
        let signed_xml = String::from_utf8(output.stdout)
            .map_err(|e| format!("Error decodificando salida UTF-8 del firmador: {}", e))?;
        if signed_xml.contains("<ds:Signature") {
            Ok(signed_xml)
        } else {
            Err("El firmador Java no devolvió un bloque ds:Signature válido".to_string())
        }
    } else {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        Err(format!("Error en el firmador Java: {}", err_msg))
    }
}

fn firmar_xml_rust(
    p12_bytes: &[u8],
    password: &str,
    xml_content: &str,
) -> Result<String, String> {
    let _legacy_prov = Provider::load(None, "legacy");
    let _default_prov = Provider::load(None, "default");

    let pkcs12 = Pkcs12::from_der(p12_bytes)
        .map_err(|e| format!("Error al leer el archivo .p12: {}", e))?;

    let parsed = pkcs12
        .parse2(password)
        .map_err(|e| format!("Contraseña incorrecta o archivo .p12 inválido: {}", e))?;

    let pkey: PKey<openssl::pkey::Private> = parsed
        .pkey
        .ok_or_else(|| "No se encontró la clave privada en el archivo .p12".to_string())?;

    let cert: X509 = parsed
        .cert
        .ok_or_else(|| "No se encontró el certificado X509 en el archivo .p12".to_string())?;

    let cert_der = cert
        .to_der()
        .map_err(|e| format!("Error convirtiendo certificado a DER: {}", e))?;
    let cert_base64 = base64::engine::general_purpose::STANDARD.encode(&cert_der);

    let cert_digest_bytes = Sha256::digest(&cert_der);
    let cert_digest_b64 = base64::engine::general_purpose::STANDARD.encode(cert_digest_bytes);

    let issuer_name = cert.issuer_name();
    let issuer_str = format_x509_name(issuer_name);
    let serial_str = cert
        .serial_number()
        .to_bn()
        .map_err(|e| e.to_string())?
        .to_dec_str()
        .map_err(|e| e.to_string())?;

    let uuid_str = Uuid::new_v4().to_string();
    let signing_time = Local::now().format("%Y-%m-%dT%H:%M:%S%.3f-05:00").to_string();

    let factura_content = match xml_content.find("<factura") {
        Some(pos) => &xml_content[pos..],
        None => xml_content,
    };
    let c14n_xml = c14n_canonicalize(factura_content);
    let xml_digest_bytes = Sha256::digest(c14n_xml.as_bytes());
    let xml_digest_b64 = base64::engine::general_purpose::STANDARD.encode(xml_digest_bytes);

    let signed_properties_inner = format!(
        "<xades:SignedSignatureProperties>\n\
<xades:SigningTime>{}</xades:SigningTime>\n\
<xades:SigningCertificate>\n\
<xades:Cert>\n\
<xades:CertDigest>\n\
<ds:DigestMethod Algorithm=\"http://www.w3.org/2001/04/xmlenc#sha256\"/>\n\
<ds:DigestValue>{}</ds:DigestValue>\n\
</xades:CertDigest>\n\
<xades:IssuerSerial>\n\
<ds:X509IssuerName>{}</ds:X509IssuerName>\n\
<ds:X509SerialNumber>{}</ds:X509SerialNumber>\n\
</xades:IssuerSerial>\n\
</xades:Cert>\n\
</xades:SigningCertificate>\n\
</xades:SignedSignatureProperties>\n\
<xades:SignedDataObjectProperties>\n\
<xades:DataObjectFormat ObjectReference=\"#xmldsig-{}-ref0\">\n\
<xades:Description>FIRMA DIGITAL SRI</xades:Description>\n\
<xades:MimeType>text/xml</xades:MimeType>\n\
<xades:Encoding>UTF-8</xades:Encoding>\n\
</xades:DataObjectFormat>\n\
</xades:SignedDataObjectProperties>",
        signing_time, cert_digest_b64, issuer_str, serial_str, uuid_str
    );

    let signed_properties_c14n = format!(
        "<xades:SignedProperties xmlns:ds=\"http://www.w3.org/2000/09/xmldsig#\" xmlns:xades=\"http://uri.etsi.org/01903/v1.3.2#\" xmlns:xades141=\"http://uri.etsi.org/01903/v1.4.1#\" Id=\"xmldsig-{}-signedprops\">\n\
{}\n\
</xades:SignedProperties>",
        uuid_str, signed_properties_inner
    );

    let signed_properties_doc = format!(
        "<xades:SignedProperties Id=\"xmldsig-{}-signedprops\">\n\
{}\n\
</xades:SignedProperties>",
        uuid_str, signed_properties_inner
    );

    let signed_properties_digest_bytes = Sha256::digest(signed_properties_c14n.as_bytes());
    let signed_properties_digest_b64 = base64::engine::general_purpose::STANDARD.encode(signed_properties_digest_bytes);

    let signed_info_inner = format!(
        "<ds:CanonicalizationMethod Algorithm=\"http://www.w3.org/TR/2001/REC-xml-c14n-20010315\"/>\n\
<ds:SignatureMethod Algorithm=\"http://www.w3.org/2001/04/xmldsig-more#rsa-sha256\"/>\n\
<ds:Reference Id=\"xmldsig-{}-ref0\" URI=\"#comprobante\">\n\
<ds:Transforms>\n\
<ds:Transform Algorithm=\"http://www.w3.org/2000/09/xmldsig#enveloped-signature\"/>\n\
</ds:Transforms>\n\
<ds:DigestMethod Algorithm=\"http://www.w3.org/2001/04/xmlenc#sha256\"/>\n\
<ds:DigestValue>{}</ds:DigestValue>\n\
</ds:Reference>\n\
<ds:Reference Type=\"http://uri.etsi.org/01903#SignedProperties\" URI=\"#xmldsig-{}-signedprops\">\n\
<ds:DigestMethod Algorithm=\"http://www.w3.org/2001/04/xmlenc#sha256\"/>\n\
<ds:DigestValue>{}</ds:DigestValue>\n\
</ds:Reference>",
        uuid_str, xml_digest_b64, uuid_str, signed_properties_digest_b64
    );

    let signed_info_c14n = format!(
        "<ds:SignedInfo xmlns:ds=\"http://www.w3.org/2000/09/xmldsig#\">\n\
{}\n\
</ds:SignedInfo>",
        signed_info_inner
    );

    let signed_info_doc = format!(
        "<ds:SignedInfo>\n\
{}\n\
</ds:SignedInfo>",
        signed_info_inner
    );

    let mut signer = Signer::new(MessageDigest::sha256(), &pkey)
        .map_err(|e| format!("Error creando firmador RSA: {}", e))?;
    signer
        .update(signed_info_c14n.as_bytes())
        .map_err(|e| format!("Error alimentando datos al firmador: {}", e))?;
    let signature_bytes = signer
        .sign_to_vec()
        .map_err(|e| format!("Error al firmar con clave privada: {}", e))?;
    let signature_b64 = base64::engine::general_purpose::STANDARD.encode(signature_bytes);

    let signature_block = format!(
        "<ds:Signature xmlns:ds=\"http://www.w3.org/2000/09/xmldsig#\" Id=\"xmldsig-{}\">\n\
{}\n\
<ds:SignatureValue Id=\"xmldsig-{}-sigvalue\">{}</ds:SignatureValue>\n\
<ds:KeyInfo>\n\
<ds:X509Data>\n\
<ds:X509Certificate>{}</ds:X509Certificate>\n\
</ds:X509Data>\n\
</ds:KeyInfo>\n\
<ds:Object>\n\
<xades:QualifyingProperties xmlns:xades=\"http://uri.etsi.org/01903/v1.3.2#\" xmlns:xades141=\"http://uri.etsi.org/01903/v1.4.1#\" Target=\"#xmldsig-{}\">\n\
{}\n\
</xades:QualifyingProperties>\n\
</ds:Object>\n\
</ds:Signature>\n",
        uuid_str,
        signed_info_doc,
        uuid_str,
        signature_b64,
        cert_base64,
        uuid_str,
        signed_properties_doc
    );

    let xml_firmado = match xml_content.rfind("</factura>") {
        Some(pos) => format!("{}{}</factura>", &xml_content[..pos], signature_block),
        None => return Err("XML inválido: no se encontró la etiqueta de cierre </factura>".to_string()),
    };

    Ok(xml_firmado)
}

fn c14n_canonicalize(xml: &str) -> String {
    xml.replace("\r\n", "\n").replace('\r', "\n")
}

fn format_x509_name(name: &openssl::x509::X509NameRef) -> String {
    let mut parts = Vec::new();
    let entries: Vec<_> = name.entries().collect();
    for entry in entries.into_iter().rev() {
        let object = entry.object();
        let nid = object.nid();
        let sn = match nid.short_name() {
            Ok("commonName") | Ok("CN") => "CN",
            Ok("organizationName") | Ok("O") => "O",
            Ok("organizationalUnitName") | Ok("OU") => "OU",
            Ok("countryName") | Ok("C") => "C",
            Ok("localityName") | Ok("L") => "L",
            Ok("stateOrProvinceName") | Ok("ST") => "ST",
            Ok("streetAddress") | Ok("STREET") => "STREET",
            Ok("serialNumber") | Ok("SN") => "SERIALNUMBER",
            Ok(other) => other,
            Err(_) => "UNKNOWN",
        };
        if let Ok(value) = entry.data().to_string() {
            parts.push(format!("{}={}", sn, value));
        }
    }
    parts.join(",")
}
