package sri;

import javax.xml.crypto.dsig.*;
import javax.xml.crypto.dsig.dom.DOMSignContext;
import javax.xml.crypto.dsig.keyinfo.*;
import javax.xml.crypto.dsig.spec.*;
import java.io.*;
import java.security.*;
import java.security.cert.X509Certificate;
import java.text.SimpleDateFormat;
import java.util.*;
import javax.xml.parsers.*;
import javax.xml.transform.*;
import javax.xml.transform.dom.DOMSource;
import javax.xml.transform.stream.StreamResult;
import org.w3c.dom.*;

public class SriSigner {
    public static void main(String[] args) {
        try {
            if (args.length < 3) {
                System.err.println("Usage: java sri.SriSigner <p12_path> <password> <xml_path>");
                System.exit(1);
            }
            String p12Path = args[0];
            String password = args[1];
            String xmlPath = args[2];

            // 1. Cargar P12
            KeyStore ks = KeyStore.getInstance("PKCS12");
            try (FileInputStream fis = new FileInputStream(p12Path)) {
                ks.load(fis, password.toCharArray());
            }

            String alias = ks.aliases().nextElement();
            PrivateKey privateKey = (PrivateKey) ks.getKey(alias, password.toCharArray());
            X509Certificate cert = (X509Certificate) ks.getCertificate(alias);

            // 2. Cargar XML
            DocumentBuilderFactory dbf = DocumentBuilderFactory.newInstance();
            dbf.setNamespaceAware(true);
            Document doc;
            try (FileInputStream fis = new FileInputStream(xmlPath)) {
                doc = dbf.newDocumentBuilder().parse(fis);
            }

            Element root = doc.getDocumentElement();
            // Registrar explícitamente el atributo 'id' como un ID XML reconocible por Java DOM
            if (root.hasAttribute("id")) {
                root.setIdAttribute("id", true);
            }

            String uuid = UUID.randomUUID().toString();
            String sigId = "xmldsig-" + uuid;
            String ref0Id = sigId + "-ref0";
            String signedPropsId = sigId + "-signedprops";

            // 3. SHA-256 Digest del Certificado
            MessageDigest md = MessageDigest.getInstance("SHA-256");
            byte[] certDigest = md.digest(cert.getEncoded());
            String certDigestB64 = Base64.getEncoder().encodeToString(certDigest);

            String issuerName = cert.getIssuerX500Principal().getName(javax.security.auth.x500.X500Principal.RFC2253);
            String serialNumber = cert.getSerialNumber().toString();

            SimpleDateFormat sdf = new SimpleDateFormat("yyyy-MM-dd'T'HH:mm:ss.SSSXXX");
            String signingTime = sdf.format(new Date());

            // 4. Crear el elemento <xades:QualifyingProperties> en DOM
            String xadesNs = "http://uri.etsi.org/01903/v1.3.2#";
            String xades141Ns = "http://uri.etsi.org/01903/v1.4.1#";
            String dsNs = "http://www.w3.org/2000/09/xmldsig#";

            Element qp = doc.createElementNS(xadesNs, "xades:QualifyingProperties");
            qp.setAttributeNS("http://www.w3.org/2000/xmlns/", "xmlns:xades141", xades141Ns);
            qp.setAttribute("Target", "#" + sigId);

            Element sp = doc.createElementNS(xadesNs, "xades:SignedProperties");
            sp.setAttribute("Id", signedPropsId);
            sp.setIdAttribute("Id", true);

            Element ssp = doc.createElementNS(xadesNs, "xades:SignedSignatureProperties");
            Element st = doc.createElementNS(xadesNs, "xades:SigningTime");
            st.setTextContent(signingTime);
            ssp.appendChild(st);

            Element sc = doc.createElementNS(xadesNs, "xades:SigningCertificate");
            Element c = doc.createElementNS(xadesNs, "xades:Cert");

            Element cd = doc.createElementNS(xadesNs, "xades:CertDigest");
            Element dm = doc.createElementNS(dsNs, "ds:DigestMethod");
            dm.setAttribute("Algorithm", "http://www.w3.org/2001/04/xmlenc#sha256");
            Element dv = doc.createElementNS(dsNs, "ds:DigestValue");
            dv.setTextContent(certDigestB64);
            cd.appendChild(dm);
            cd.appendChild(dv);
            c.appendChild(cd);

            Element is = doc.createElementNS(xadesNs, "xades:IssuerSerial");
            Element xn = doc.createElementNS(dsNs, "ds:X509IssuerName");
            xn.setTextContent(issuerName);
            Element sn = doc.createElementNS(dsNs, "ds:X509SerialNumber");
            sn.setTextContent(serialNumber);
            is.appendChild(xn);
            is.appendChild(sn);
            c.appendChild(is);

            sc.appendChild(c);
            ssp.appendChild(sc);
            sp.appendChild(ssp);

            Element sdop = doc.createElementNS(xadesNs, "xades:SignedDataObjectProperties");
            Element dof = doc.createElementNS(xadesNs, "xades:DataObjectFormat");
            dof.setAttribute("ObjectReference", "#" + ref0Id);
            Element desc = doc.createElementNS(xadesNs, "xades:Description");
            desc.setTextContent("FIRMA DIGITAL SRI");
            Element mt = doc.createElementNS(xadesNs, "xades:MimeType");
            mt.setTextContent("text/xml");
            Element enc = doc.createElementNS(xadesNs, "xades:Encoding");
            enc.setTextContent("UTF-8");
            dof.appendChild(desc);
            dof.appendChild(mt);
            dof.appendChild(enc);
            sdop.appendChild(dof);
            sp.appendChild(sdop);

            qp.appendChild(sp);

            // 5. XMLSignatureFactory
            XMLSignatureFactory fac = XMLSignatureFactory.getInstance("DOM");

            // Referencia 1: #comprobante
            Reference refComprobante = fac.newReference(
                "#comprobante",
                fac.newDigestMethod(DigestMethod.SHA256, null),
                Collections.singletonList(fac.newTransform(Transform.ENVELOPED, (TransformParameterSpec) null)),
                null,
                ref0Id
            );

            // Referencia 2: #signedprops
            Reference refSignedProps = fac.newReference(
                "#" + signedPropsId,
                fac.newDigestMethod(DigestMethod.SHA256, null),
                null,
                "http://uri.etsi.org/01903#SignedProperties",
                null
            );

            List<Reference> references = Arrays.asList(refComprobante, refSignedProps);

            SignedInfo si = fac.newSignedInfo(
                fac.newCanonicalizationMethod(CanonicalizationMethod.INCLUSIVE, (C14NMethodParameterSpec) null),
                fac.newSignatureMethod("http://www.w3.org/2001/04/xmldsig-more#rsa-sha256", null),
                references
            );

            // KeyInfo
            KeyInfoFactory kif = fac.getKeyInfoFactory();
            X509Data x509Data = kif.newX509Data(Collections.singletonList(cert));
            KeyInfo ki = kif.newKeyInfo(Collections.singletonList(x509Data));

            // ds:Object
            XMLObject object = fac.newXMLObject(Collections.singletonList(new javax.xml.crypto.dom.DOMStructure(qp)), null, null, null);

            // Crear Firma Completa
            XMLSignature signature = fac.newXMLSignature(si, ki, Collections.singletonList(object), sigId, null);

            DOMSignContext dsc = new DOMSignContext(privateKey, root);
            dsc.setDefaultNamespacePrefix("ds");
            dsc.setIdAttributeNS(root, null, "id");
            dsc.setIdAttributeNS(sp, null, "Id");

            signature.sign(dsc);

            // Exportar XML Firmado
            TransformerFactory tf = TransformerFactory.newInstance();
            Transformer trans = tf.newTransformer();
            trans.setOutputProperty(OutputKeys.OMIT_XML_DECLARATION, "no");
            trans.setOutputProperty(OutputKeys.ENCODING, "UTF-8");
            trans.transform(new DOMSource(doc), new StreamResult(System.out));

        } catch (Exception e) {
            e.printStackTrace(System.err);
            System.exit(1);
        }
    }
}
