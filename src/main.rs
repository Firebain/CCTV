use xml::writer::{EventWriter, EmitterConfig, XmlEvent, Error};
use std::io::Write;

fn main() {
    let probe = build_probe("Device", "2cdcc10c-b8d1-441f-8a48-d1da20dcdea1").unwrap();

    println!("{}", probe);
}

// <s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope" xmlns:a="http://schemas.xmlsoap.org/ws/2004/08/addressing">
//     <s:Header>
//         <a:Action s:mustUnderstand="1">http://schemas.xmlsoap.org/ws/2005/04/discovery/Probe</a:Action>
//         <a:MessageID>uuid:{uuid}</a:MessageID>
//         <a:ReplyTo>
//             <a:Address>http://schemas.xmlsoap.org/ws/2004/08/addressing/role/anonymous</a:Address>
//         </a:ReplyTo>
//         <a:To s:mustUnderstand="1">urn:schemas-xmlsoap-org:ws:2005:04:discovery</a:To>
//     </s:Header>
//     <s:Body>
//         <Probe xmlns="http://schemas.xmlsoap.org/ws/2005/04/discovery">
//             <d:Types xmlns:d="http://schemas.xmlsoap.org/ws/2005/04/discovery" 
//                 xmlns:dp0="http://www.onvif.org/ver10/network/wsdl">
//                     dp0:{type}
//             </d:Types>
//         </Probe>
//     </s:Body>
// </s:Envelope>

fn build_probe(device_type: &str, uuid: &str) -> Result<String, Error> {
    let mut buffer = Vec::new();

    let mut writer = EmitterConfig::new()
        .perform_indent(true)
        .create_writer(&mut buffer);

    envelope(&mut writer, |writer| {
        header(writer, move |writer| {
            action(writer);

            message_id(writer, uuid);

            reply_to(writer, |writer| {
                address(writer);
            });

            to(writer);
        });

        body(writer, |writer| {
            probe(writer, move |writer| {
                types(writer, device_type);
            });
        });
    });

    Ok(String::from_utf8(buffer).unwrap())
}

fn envelope<W, F>(writer: &mut EventWriter<W>, mut func: F) where
    W: Write,
    F: FnMut(&mut EventWriter<W>)
{
    let start = XmlEvent::start_element("s:Envelope")
        .ns("a", "http://schemas.xmlsoap.org/ws/2004/08/addressing")
        .ns("s", "http://www.w3.org/2003/05/soap-envelope");

    writer.write(start).unwrap();

    func(writer);

    let end = XmlEvent::end_element();

    writer.write(end).unwrap();
}

fn header<W, F>(writer: &mut EventWriter<W>, mut func: F) where
    W: Write,
    F: FnMut(&mut EventWriter<W>) 
{
    let start = XmlEvent::start_element("s:Header");

    writer.write(start).unwrap();

    func(writer);

    let end = XmlEvent::end_element();

    writer.write(end).unwrap();
}

fn action<W: Write>(writer: &mut EventWriter<W>) {
    let start = XmlEvent::start_element("a:Action")
        .attr("s:mustUnderstand", "1");

    writer.write(start).unwrap();

    let content = XmlEvent::characters("http://schemas.xmlsoap.org/ws/2005/04/discovery/Probe");

    writer.write(content).unwrap();

    let end = XmlEvent::end_element();

    writer.write(end).unwrap();
}

fn message_id<W: Write>(writer: &mut EventWriter<W>, uuid: &str) {
    let start = XmlEvent::start_element("a:MessageID");

    writer.write(start).unwrap();

    let text = format!("uuid:{}", uuid);

    let content = XmlEvent::characters(&text);

    writer.write(content).unwrap();

    let end = XmlEvent::end_element();

    writer.write(end).unwrap();
}

fn reply_to<W, F>(writer: &mut EventWriter<W>, mut func: F) where
    W: Write,
    F: FnMut(&mut EventWriter<W>) 
{
    let start = XmlEvent::start_element("a:ReplyTo");

    writer.write(start).unwrap();

    func(writer);

    let end = XmlEvent::end_element();

    writer.write(end).unwrap();
}

fn address<W: Write>(writer: &mut EventWriter<W>) {
    let start = XmlEvent::start_element("a:Address");

    writer.write(start).unwrap();

    let content = XmlEvent::characters("http://schemas.xmlsoap.org/ws/2004/08/addressing/role/anonymous");

    writer.write(content).unwrap();

    let end = XmlEvent::end_element();

    writer.write(end).unwrap();
}

fn to<W: Write>(writer: &mut EventWriter<W>) {
    let start = XmlEvent::start_element("a:To")
        .attr("s:mustUnderstand", "1");

    writer.write(start).unwrap();

    let content = XmlEvent::characters("urn:schemas-xmlsoap-org:ws:2005:04:discovery");

    writer.write(content).unwrap();

    let end = XmlEvent::end_element();

    writer.write(end).unwrap();
}

fn body<W, F>(writer: &mut EventWriter<W>, mut func: F) where
    W: Write,
    F: FnMut(&mut EventWriter<W>) 
{
    let start = XmlEvent::start_element("s:Body");

    writer.write(start).unwrap();

    func(writer);

    let end = XmlEvent::end_element();

    writer.write(end).unwrap();
}

fn probe<W, F>(writer: &mut EventWriter<W>, mut func: F) where
    W: Write,
    F: FnMut(&mut EventWriter<W>) 
{
    let start = XmlEvent::start_element("Probe")
        .default_ns("http://schemas.xmlsoap.org/ws/2005/04/discovery");

    writer.write(start).unwrap();

    func(writer);

    let end = XmlEvent::end_element();

    writer.write(end).unwrap();
}

fn types<W: Write>(writer: &mut EventWriter<W>, device_type: &str) {
    let start = XmlEvent::start_element("d:Types")
        .ns("d", "http://schemas.xmlsoap.org/ws/2005/04/discovery")
        .ns("dp0", "http://www.onvif.org/ver10/network/wsdl");;

    writer.write(start).unwrap();

    let text = format!("dp0:{}", device_type);

    let content = XmlEvent::characters(&text);

    writer.write(content).unwrap();

    let end = XmlEvent::end_element();

    writer.write(end).unwrap();
}

// fn probe(device_type: &str, uuid: &str) -> Result<String, Error> {
//     let mut buffer = Vec::new();

//     let mut writer = EmitterConfig::new()
//         .perform_indent(true)
//         .create_writer(&mut buffer);

    // let envelope_start = XmlEvent::start_element("s:Envelope")
    //     .ns("a", "http://schemas.xmlsoap.org/ws/2004/08/addressing")
    //     .ns("s", "http://www.w3.org/2003/05/soap-envelope");

//     let envelope_end = XmlEvent::end_element();

//     let header_start = XmlEvent::start_element("s:Header");

//     let header_end = XmlEvent::end_element();

//     let action_start = XmlEvent::start_element("a:Action")
//         .attr("s:mustUnderstand", "1");

//     let action_end = XmlEvent::end_element();

//     let action_content = XmlEvent::characters("http://schemas.xmlsoap.org/ws/2005/04/discovery/Probe");

//     let message_id_start = XmlEvent::start_element("a:MessageID");

//     let message_id_end = XmlEvent::end_element();

//     let message_id_text = format!("uuid:{}", uuid);

//     let message_id_content = XmlEvent::characters(&message_id_text);

//     let reply_to_start = XmlEvent::start_element("a:ReplyTo");

//     let reply_to_end = XmlEvent::end_element();

//     let address_start = XmlEvent::start_element("a:Address");

//     let address_end = XmlEvent::end_element();

//     let address_content = XmlEvent::characters("http://schemas.xmlsoap.org/ws/2004/08/addressing/role/anonymous");

//     let to_start = XmlEvent::start_element("a:To")
//         .attr("s:mustUnderstand", "1");

//     let to_end = XmlEvent::end_element();

//     let to_content = XmlEvent::characters("urn:schemas-xmlsoap-org:ws:2005:04:discovery");

//     let body_start = XmlEvent::start_element("s:Body");

//     let body_end = XmlEvent::end_element();

//     let probe_start = XmlEvent::start_element("Probe")
//         .default_ns("http://schemas.xmlsoap.org/ws/2005/04/discovery");

//     let probe_end = XmlEvent::end_element();

//     let types_start = XmlEvent::start_element("d:Types")
//         .ns("d", "http://schemas.xmlsoap.org/ws/2005/04/discovery")
//         .ns("dp0", "http://www.onvif.org/ver10/network/wsdl");

//     let types_end = XmlEvent::end_element();

//     let types_text = format!("dp0:{}", device_type);

//     let types_content = XmlEvent::characters(&types_text);

//     writer.write(envelope_start).unwrap();

//     writer.write(header_start).unwrap();

//     writer.write(action_start).unwrap();

//     writer.write(action_content).unwrap();

//     writer.write(action_end).unwrap();

//     writer.write(message_id_start).unwrap();

//     writer.write(message_id_content).unwrap();

//     writer.write(message_id_end).unwrap();

//     writer.write(reply_to_start).unwrap();

//     writer.write(address_start).unwrap();

//     writer.write(address_content).unwrap();

//     writer.write(address_end).unwrap();

//     writer.write(reply_to_end).unwrap();

//     writer.write(to_start).unwrap();

//     writer.write(to_content).unwrap();

//     writer.write(to_end).unwrap();

//     writer.write(header_end).unwrap();

//     writer.write(body_start).unwrap();

//     writer.write(probe_start).unwrap();

//     writer.write(types_start).unwrap();

//     writer.write(types_content).unwrap();

//     writer.write(types_end).unwrap();

//     writer.write(probe_end).unwrap();

//     writer.write(body_end).unwrap();

//     writer.write(envelope_end).unwrap();

//     Ok(String::from_utf8(buffer).unwrap())
// }