/* 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2021 Robert D. French
 */


use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::take_till1;
use nom::bytes::complete::take_while;
use nom::combinator::map as nom_map;
use nom::combinator::recognize;
use nom::combinator::value;
use nom::IResult;
use nom::sequence::tuple;
use std::net::AddrParseError;
use std::net::SocketAddr;
use std::path::Path;


pub fn sp<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
    let chars = " \t\r\n";

    take_while(move |c| chars.contains(c))(input)
}


mod keyword {

    #[derive(Debug, PartialEq, Clone)]
    pub struct To {}

    #[derive(Debug, PartialEq, Clone)]
    pub struct Map {}
}

pub fn to<'a>(input: &'a str) -> IResult<&'a str, keyword::To> {
    value(keyword::To{}, tag("to"))(input)
}

pub fn map<'a>(input: &'a str) -> IResult<&'a str, keyword::Map> {
    value(keyword::Map{}, tag("map"))(input)
}


#[derive(Debug, PartialEq, Clone)]
pub enum HttpMethod {
    GET,
    DELETE,
    POST,
    PUT
}

pub fn http_method<'a>(input: &'a str) -> IResult<&'a str, HttpMethod> {
    alt((
        value(HttpMethod::GET, tag("GET")),
        value(HttpMethod::DELETE, tag("DELETE")),
        value(HttpMethod::POST, tag("POST")),
        value(HttpMethod::PUT, tag("PUT"))
    ))(input)
}

pub fn contiguous_text<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
    let till_whitespace = take_till1(|c: char| c.is_whitespace());
    recognize(till_whitespace)(input)
}

pub fn path<'a>(input: &'a str) -> IResult<&'a str, &'a Path> {
    nom_map(contiguous_text, |p: &str| Path::new(p))(input)
}

pub fn socket_address<'a>(input: &'a str) -> Result<SocketAddr, AddrParseError> {
    input.parse()
}

#[derive(Debug,PartialEq,Clone)]
pub struct MapStatement<'a> {
    pub method: HttpMethod,
    pub url_prefix: &'a Path,
    pub door: &'a Path
}

pub fn map_statement<'a>(input: &'a str) -> IResult<&'a str, MapStatement<'a>> {
    let (input, (_map, _sp1, method, _sp2, url_prefix, _sp3, _to, _sp4, door)) = tuple((
            map,sp,http_method,sp,path,sp,to,sp,path
    ))(input)?;
    Ok((input, MapStatement{ method, url_prefix, door }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_space() {
        assert_eq!(sp(" "), Ok((""," ")))
    }

    #[test]
    fn detect_to() {
        assert_eq!(to("to"), Ok(("",keyword::To{})))
    }

    #[test]
    fn detect_map() {
        assert_eq!(map("map"), Ok(("",keyword::Map{})))
    }

    #[test]
    fn detect_http_method() {
        assert_eq!(http_method("GET"), Ok(("",HttpMethod::GET)));
        assert_eq!(http_method("DELETE"), Ok(("",HttpMethod::DELETE)));
        assert_eq!(http_method("POST"), Ok(("",HttpMethod::POST)));
        assert_eq!(http_method("PUT"), Ok(("",HttpMethod::PUT)))
    }

    #[test]
    fn detect_contiguous_text() {
        assert_eq!(contiguous_text("/my/path /not/my/path"), Ok((" /not/my/path","/my/path")));
        assert_eq!(contiguous_text("0.0.0.0:80 garbage"), Ok((" garbage","0.0.0.0:80")));
        assert_eq!(contiguous_text("var1 var2 var3"), Ok((" var2 var3","var1")));
    }

    #[test]
    fn detect_path() {
        assert_eq!(path("/my/path NOT"), Ok((" NOT",Path::new("/my/path"))));
    }

    #[test]
    fn parse_map_statement() {
        let actual = map_statement("map GET /uri to /my/door");
        let expected = MapStatement{
            door: Path::new("/my/door"),
            url_prefix: Path::new("/uri"),
            method: HttpMethod::GET
        };
        assert_eq!(actual, Ok(("",expected)))
    }
}
