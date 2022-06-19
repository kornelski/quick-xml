#![allow(unused)]

use quick_xml::events::Event;
use quick_xml::name::Namespace;
use quick_xml::Reader;
use std::convert::TryFrom;
use std::io::Read;

struct Resource {
    etag: String,
    calendar_data: String,
}

struct Prop {
    namespace: String,
    local_name: String,
    value: String,
}

impl Prop {
    fn new() -> Prop {
        Prop {
            namespace: String::new(),
            local_name: String::new(),
            value: String::new(),
        }
    }
}

struct PropStat {
    status: String,
    props: Vec<Prop>,
}

impl PropStat {
    fn new() -> PropStat {
        PropStat {
            status: String::new(),
            props: Vec::<Prop>::new(),
        }
    }
}

struct Response {
    href: String,
    propstats: Vec<PropStat>,
}

impl Response {
    fn new() -> Response {
        Response {
            href: String::new(),
            propstats: Vec::<PropStat>::new(),
        }
    }
}

fn parse_report(xml_data: &str) -> Vec<Resource> {
    let result = Vec::<Resource>::new();

    let mut reader = Reader::from_str(xml_data);
    reader.trim_text(true);

    let mut count = 0;
    let mut buf = Vec::new();
    let mut ns_buffer = Vec::new();

    #[derive(Clone, Copy)]
    enum State {
        Root,
        MultiStatus,
        Response,
        Success,
        Error,
    };

    let mut responses = Vec::<Response>::new();
    let mut current_response = Response::new();
    let mut reading_href = false;
    let mut current_prop = Prop::new();

    let mut depth = 0;
    let mut state = State::MultiStatus;

    loop {
        match reader.read_namespaced_event(&mut buf, &mut ns_buffer) {
            Ok((ns, Event::Start(e))) => {
                let ns = Option::<Namespace>::try_from(ns)
                    .unwrap_or_default() // Treat unknown prefixes as not bound to any namespace
                    .unwrap_or(Namespace(b""));
                match (depth, state, ns.as_ref(), e.local_name().as_ref()) {
                    (0, State::Root, b"DAV:", b"multistatus") => state = State::MultiStatus,
                    (1, State::MultiStatus, b"DAV:", b"response") => {
                        state = State::Response;
                        current_response = Response::new();
                    }
                    (2, State::Response, b"DAV:", b"href") => {
                        current_response.href.clear();
                        reading_href = true;
                    }
                    _ => {}
                }
                depth += 1;
            }
            Ok((ns, Event::End(e))) => {
                reading_href = false;
                let ns = Option::<Namespace>::try_from(ns)
                    .unwrap_or_default() // Treat unknown prefixes as not bound to any namespace
                    .unwrap_or(Namespace(b""));
                match (depth, state, ns.as_ref(), e.local_name().as_ref()) {
                    (1, State::MultiStatus, b"DAV:", b"multistatus") => state = State::Root,
                    (2, State::MultiStatus, b"DAV:", b"multistatus") => state = State::MultiStatus,
                    _ => {}
                }
                depth -= 1;
            }
            Ok((_, Event::Text(e))) if reading_href => {
                current_response.href.push_str(&e.unescape_and_decode(&reader).unwrap());
            }
            Ok((_, Event::Eof)) => break,
            Err(e) => break,
            _ => (),
        }
    }
    result
}

fn main() {
    let test_data = r#"
<?xml version="1.0" encoding="UTF-8"?>
<D:multistatus xmlns:D="DAV:" xmlns:caldav="urn:ietf:params:xml:ns:caldav"
    xmlns:cs="http://calendarserver.org/ns/" xmlns:ical="http://apple.com/ns/ical/">
 <D:response xmlns:carddav="urn:ietf:params:xml:ns:carddav"
    xmlns:cm="http://cal.me.com/_namespace/" xmlns:md="urn:mobileme:davservices">
  <D:href>
  /caldav/v2/johndoh%40gmail.com/events/07b7it7uonpnlnvjldr0l1ckg8%40google.com.ics
  </D:href>
  <D:propstat>
   <D:status>HTTP/1.1 200 OK</D:status>
   <D:prop>
    <D:getetag>"63576798396"</D:getetag>
    <caldav:calendar-data>BEGIN:VCALENDAR</caldav:calendar-data>
   </D:prop>
  </D:propstat>
 </D:response>
</D:multistatus>
"#;

    parse_report(test_data);
}
