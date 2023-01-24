use std::{error::Error, io::Cursor};
use std::fmt::Display;
use std::io::{Read, Write};
use std::time::Duration;

use curl::easy::{Easy, List};
use xmltree::Element;
use xml::writer::{EventWriter, XmlEvent};

const WIRESERVER: &str = "http://168.63.129.16";

#[derive(Debug)]
enum AppError {
    Io(std::io::Error),
    Curl(curl::Error),
    ParseXml(xmltree::ParseError),
    WriteXml(xml::writer::Error),
    Utf8(std::string::FromUtf8Error),
    Data(String),
}

type AppResult<T> = Result<T, AppError>;

#[derive(Debug)]
struct GoalState {
    container_id: String,
    instance_id: String,
    incarnation: String,
}

fn main() -> AppResult<()> {
    eprintln!("Fetching goal state...");
    let goal_state_buf = retry(get_goal_state)?;
    let goal_state = parse_goal_state(&String::from_utf8(goal_state_buf)?)?;
    eprintln!("{:?}", goal_state);

    let ready_data = get_ready_data(&goal_state)?;
    eprintln!("Reporting readiness: {}", String::from_utf8(ready_data.clone())?);
    retry(|| report_ready(&ready_data))?;
    eprintln!("Reported ready, exiting");
    Ok(())
}

fn get_goal_state() -> AppResult<Vec<u8>> {
    let mut request = Easy::new();
    request.url(&format!("{}/machine?comp=goalstate", WIRESERVER)).unwrap();
    let mut headers = List::new();
    headers.append("x-ms-version: 2012-11-30").unwrap();
    request.http_headers(headers).unwrap();

    let mut buf = Vec::new();
    let mut transfer = request.transfer();
    transfer.write_function(|data| {
        buf.extend_from_slice(data);
        Ok(data.len())
    })?;
    transfer.perform()?;
    drop(transfer);

    let response_code = request.response_code()?;
    if response_code != 200 {
        return Err(AppError::Data(format!("HTTP status code {} while retrieving goal state",
                                          response_code)));
    }
    Ok(buf)
}

fn parse_goal_state(xml: &str) -> AppResult<GoalState> {
    let root = Element::parse(xml.as_bytes())?;
    let container = get_element(&root, "Container")?;

    let incarnation = get_element_text(&root, "Incarnation")?;
    let container_id = get_element_text(container, "ContainerId")?;

    let role_instance_list = get_element(container, "RoleInstanceList")?;
    let role_instance = get_element(role_instance_list, "RoleInstance")?;
    let instance_id = get_element_text(role_instance, "InstanceId")?;

    Ok(GoalState {
        container_id,
        instance_id,
        incarnation,
    })
}

fn get_ready_data(goal_state: &GoalState) -> AppResult<Vec<u8>> {
    let cursor = Cursor::new(Vec::new());
    let mut w = EventWriter::new(cursor);

    w.write(XmlEvent::start_element("Health"))?;
    write_tag(&mut w, "GoalStateIncarnation", &goal_state.incarnation)?;

    w.write(XmlEvent::start_element("Container"))?;
        write_tag(&mut w, "ContainerId", &goal_state.container_id)?;

        w.write(XmlEvent::start_element("RoleInstanceList"))?;
            w.write(XmlEvent::start_element("Role"))?;
                write_tag(&mut w, "InstanceId", &goal_state.instance_id)?;

                w.write(XmlEvent::start_element("Health"))?;
                    write_tag(&mut w, "State", "Ready")?;
                w.write(XmlEvent::end_element())?;
            w.write(XmlEvent::end_element())?;
        w.write(XmlEvent::end_element())?;
    w.write(XmlEvent::end_element())?;

    w.write(XmlEvent::end_element())?;
    Ok(w.into_inner().into_inner())
}

fn write_tag<W: Write>(w: &mut EventWriter<W>, tag: &str, value: &str) -> AppResult<()> {
    w.write(XmlEvent::start_element(tag))?;
    w.write(XmlEvent::characters(value))?;
    w.write(XmlEvent::end_element())?;
    Ok(())
}

fn report_ready(goal_state: &[u8]) -> AppResult<()> {
    let mut request = Easy::new();
    request.url(&format!("{}/machine?comp=health", WIRESERVER)).unwrap();
    request.post(true).unwrap();
    request.post_field_size(goal_state.len().try_into().unwrap()).unwrap();

    let mut headers = List::new();
    headers.append("x-ms-version: 2012-11-30").unwrap();
    headers.append("x-ms-agent-name: custom-provisioning").unwrap();
    headers.append("content-type: text/xml; charset=utf-8").unwrap();
    request.http_headers(headers).unwrap();

    let mut cursor = Cursor::new(goal_state);
    let mut transfer = request.transfer();
    transfer.read_function(|data| {
        cursor.read(data).map_err(|_| curl::easy::ReadError::Abort)
    })?;
    transfer.perform()?;
    drop(transfer);

    let response_code = request.response_code()?;
    if response_code != 200 {
        return Err(AppError::Data(format!("HTTP status code {} while reporting ready",
                                          response_code)));
    }
    Ok(())
}

fn retry<T, F: Fn() -> Result<T, AppError>>(f: F) -> AppResult<T> {
    let mut timeout = Duration::from_secs(2);
    const MAX_TIMEOUT: Duration = Duration::from_secs(2 * 60);

    loop {
        let res = f();
        if res.is_ok() {
            return res;
        }
        if timeout > MAX_TIMEOUT {
            eprintln!("Retry limit exceeded, aborting");
            return res;
        }
        eprintln!("{:?}", res.err().unwrap());
        std::thread::sleep(timeout);
        timeout *= 2;
    }
}

fn get_element<'e, 't>(el: &'e Element, tag: &'t str) -> AppResult<&'e Element> {
    let child = el.get_child(tag);
    child.ok_or_else(|| AppError::Data(format!("Parse goal state: Missing {} tag", tag)))
}

fn get_element_text(el: &Element, tag: &str) -> AppResult<String> {
    let child = get_element(el, tag)?;
    Ok(match child.get_text() {
        Some(s) => s.into_owned(),
        None => String::new(),
    })
}

impl Error for AppError {
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<curl::Error> for AppError {
    fn from(value: curl::Error) -> Self {
        Self::Curl(value)
    }
}

impl From<xmltree::ParseError> for AppError {
    fn from(value: xmltree::ParseError) -> Self {
        Self::ParseXml(value)
    }
}

impl From<xml::writer::Error> for AppError {
    fn from(value: xml::writer::Error) -> Self {
        Self::WriteXml(value)
    }
}

impl From<std::string::FromUtf8Error> for AppError {
    fn from(value: std::string::FromUtf8Error) -> Self {
        Self::Utf8(value)
    }
}
