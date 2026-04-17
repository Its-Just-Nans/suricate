//! The state machine that parses a char iterator of the gedcom's contents

#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::unwrap_used)]

use std::{panic, str::Chars};

use super::GedcomData;
use super::tokenizer::{Token, Tokenizer};
use super::types::{
    Address, CustomData, Event, EventType, Family, FamilyLink, FamilyLinkType, Gender, Header,
    Individual, Name, Pedigree, RepoCitation, Repository, Source, SourceCitation, Submitter,
};

/// The Gedcom parser that converts the token list into a data structure
pub struct Parser<'a> {
    tokenizer: Tokenizer<'a>,
    skip_on_error: bool,
}

type ParserError<A> = Result<A, String>;

impl<'a> Parser<'a> {
    /// Creates a parser state machine for parsing a gedcom file as a chars iterator
    #[must_use]
    pub fn new(chars: Chars<'a>) -> Parser<'a> {
        let mut tokenizer = Tokenizer::new(chars);
        tokenizer.next_token();
        Parser {
            tokenizer,
            skip_on_error: true,
        }
    }

    /// Does the actual parsing of the record.
    pub fn parse_record(&mut self) -> ParserError<GedcomData> {
        let mut data = GedcomData::default();
        loop {
            let Token::Level(level) = self.tokenizer.current_token else {
                return Err(format!(
                    "{} Expected Level, found {:?}",
                    self.dbg(),
                    self.tokenizer.current_token
                ));
            };

            self.tokenizer.next_token();
            // println!("{:?}", self.tokenizer.current_token);
            let pointer = if let Token::Pointer(xref) = &self.tokenizer.current_token {
                let xref = xref.clone();
                self.tokenizer.next_token();
                xref
            } else if let Token::Tag(tag) = &self.tokenizer.current_token
                && tag == "HEAD"
            {
                data.header = self.parse_header();
                continue;
            } else if let Token::Tag(tag) = &self.tokenizer.current_token
                && tag == "TRLR"
            {
                break;
            } else {
                return Err(format!("Unknown token: {:?}", self.tokenizer.current_token));
            };

            if let Token::Tag(tag) = &self.tokenizer.current_token {
                match tag.as_str() {
                    "FAM" => data.families.push(self.parse_family(level, Some(pointer))?),
                    "INDI" => data
                        .individuals
                        .push(self.parse_individual(level, Some(pointer))?),
                    "REPO" => data
                        .repositories
                        .push(self.parse_repository(level, Some(pointer))),
                    "SOUR" => data.sources.push(self.parse_source(level, Some(pointer))),
                    "SUBM" => data
                        .submitters
                        .push(self.parse_submitter(level, Some(pointer))?),
                    _ => {
                        if self.skip_on_error {
                            self.take_line_value();
                            continue;
                        }
                        return Err(format!("{} Unhandled tag {}", self.dbg(), tag));
                    }
                }
            } else if let Token::CustomTag(tag) = &self.tokenizer.current_token {
                // TODO
                let tag_clone = tag.clone();
                let custom_data = self.parse_custom_tag(tag_clone);
                println!(
                    "{} Skipping top-level custom tag: {:?}",
                    self.dbg(),
                    custom_data
                );
                while self.tokenizer.current_token != Token::Level(0) {
                    self.tokenizer.next_token();
                }
            } else {
                println!(
                    "{} Unhandled token {:?}",
                    self.dbg(),
                    self.tokenizer.current_token
                );
                self.tokenizer.next_token();
            }
        }

        Ok(data)
    }

    /// Parses HEAD top-level tag
    fn parse_header(&mut self) -> Header {
        // skip over HEAD tag name
        self.tokenizer.next_token();

        let mut header = Header::default();

        // just skipping the header for now
        while self.tokenizer.current_token != Token::Level(0) {
            match &self.tokenizer.current_token {
                Token::Tag(tag) => match tag.as_str() {
                    // TODO: CHAR.VERS
                    "CHAR" => header.encoding = Some(self.take_line_value()),
                    "CORP" => header.corporation = Some(self.take_line_value()),
                    "COPR" => header.copyright = Some(self.take_line_value()),
                    "DATE" => header.date = Some(self.take_line_value()),
                    "DEST" => header.destinations.push(self.take_line_value()),
                    "LANG" => header.language = Some(self.take_line_value()),
                    "FILE" => header.filename = Some(self.take_line_value()),
                    "NOTE" => header.note = Some(self.take_continued_text(1)),
                    "SUBM" => header.submitter_tag = Some(self.take_line_value()),
                    "SUBN" => header.submission_tag = Some(self.take_line_value()),
                    "TIME" => {
                        let time = self.take_line_value();
                        // assuming subtag of DATE
                        if let Some(date) = header.date {
                            let mut datetime = String::new();
                            datetime.push_str(&date);
                            datetime.push(' ');
                            datetime.push_str(&time);
                            header.date = Some(datetime);
                        } else {
                            panic!("Expected TIME to be under DATE in header.");
                        }
                    }
                    "GEDC" => {
                        header = self.parse_gedcom_data(header);
                    }
                    // TODO: HeaderSource
                    "SOUR" => {
                        println!("WARNING: Skipping header source.");
                        while self.tokenizer.current_token != Token::Level(1) {
                            self.tokenizer.next_token();
                        }
                    }
                    _ => panic!("{} Unhandled Header Tag: {}", self.dbg(), tag),
                },
                Token::Level(_) => self.tokenizer.next_token(),
                _ => panic!("Unhandled Header Token: {:?}", self.tokenizer.current_token),
            }
        }
        header
    }

    /// Parses SUBM top-level tag
    fn parse_submitter(&mut self, level: u8, xref: Option<String>) -> ParserError<Submitter> {
        // skip over SUBM tag name
        self.tokenizer.next_token();

        let mut submitter = Submitter::default();
        submitter.xref = xref;
        while self.tokenizer.current_token != Token::Level(level) {
            match &self.tokenizer.current_token {
                Token::Tag(tag) => match tag.as_str() {
                    "NAME" => submitter.name = Some(self.take_line_value()),
                    "ADDR" => {
                        submitter.address = Some(self.parse_address(level + 1));
                    }
                    "PHON" => submitter.phone = Some(self.take_line_value()),
                    "COMM" => {
                        self.take_continued_text(1);
                    }
                    a => {
                        println!("{a:?}");
                        if self.skip_on_error {
                            self.take_line_value();
                            continue;
                        }
                        return Err(format!("{} Unhandled Submitter Tag: {}", self.dbg(), tag));
                    }
                },
                Token::Level(_) => self.tokenizer.next_token(),
                _ => panic!(
                    "Unhandled Submitter Token: {:?}",
                    self.tokenizer.current_token
                ),
            }
        }
        // println!("found submitter:\n{:#?}", submitter);
        Ok(submitter)
    }

    /// Parses INDI top-level tag
    fn parse_individual(&mut self, level: u8, xref: Option<String>) -> ParserError<Individual> {
        // skip over INDI tag name
        self.tokenizer.next_token();
        let mut individual = Individual::default();
        individual.xref = xref;

        while self.tokenizer.current_token != Token::Level(level) {
            match &self.tokenizer.current_token {
                Token::Tag(tag) => match tag.as_str() {
                    "NAME" => individual.name = Some(self.parse_name(level + 1)),
                    "SEX" => individual.sex = self.parse_gender(),
                    "ADOP" | "BIRT" | "BAPM" | "BARM" | "BASM" | "BLES" | "BURI" | "CENS"
                    | "CHR" | "CHRA" | "CONF" | "CREM" | "DEAT" | "EMIG" | "FCOM" | "GRAD"
                    | "IMMI" | "NATU" | "ORDN" | "RETI" | "RESI" | "PROB" | "WILL" | "EVEN" => {
                        let tag_clone = tag.clone();
                        individual
                            .events
                            .push(self.parse_event(tag_clone.as_str(), level + 1));
                    }
                    "FAMC" | "FAMS" => {
                        let tag_clone = tag.clone();
                        individual
                            .add_family(self.parse_family_link(tag_clone.as_str(), level + 1));
                    }
                    "CHAN" => {
                        // assuming it always only has a single DATE subtag
                        self.tokenizer.next_token(); // level
                        self.tokenizer.next_token(); // DATE tag
                        individual.last_updated = Some(self.take_line_value());
                    }
                    _ => {
                        if self.skip_on_error {
                            self.take_line_value();
                            continue;
                        }
                        return Err(format!("{} Unhandled Individual Tag: {}", self.dbg(), tag));
                    }
                },
                Token::CustomTag(tag) => {
                    let tag_clone = tag.clone();
                    individual
                        .custom_data
                        .push(self.parse_custom_tag(tag_clone));
                }
                Token::Level(_) => self.tokenizer.next_token(),
                _ => panic!(
                    "Unhandled Individual Token: {:?}",
                    self.tokenizer.current_token
                ),
            }
        }
        // println!("found individual:\n{:#?}", individual);
        Ok(individual)
    }

    /// Parses FAM top-level tag
    fn parse_family(&mut self, level: u8, xref: Option<String>) -> ParserError<Family> {
        // skip over FAM tag name
        self.tokenizer.next_token();
        let mut family = Family::default();
        family.xref = xref;

        while self.tokenizer.current_token != Token::Level(level) {
            match &self.tokenizer.current_token {
                Token::Tag(tag) => match tag.as_str() {
                    "MARR" => family.events.push(self.parse_event("MARR", level + 1)),
                    "HUSB" => family.set_individual1(self.take_line_value()),
                    "WIFE" => family.set_individual2(self.take_line_value()),
                    "CHIL" => family.children.push(self.take_line_value()),
                    a => {
                        println!("{a:?}");
                        if self.skip_on_error {
                            self.take_line_value();
                            continue;
                        }
                        return Err(format!("{} Unhandled Family Tag: {}", self.dbg(), tag));
                    }
                },
                Token::Level(_) => self.tokenizer.next_token(),
                _ => {
                    return Err(format!(
                        "Unhandled Family Token: {:?}",
                        self.tokenizer.current_token
                    ));
                }
            }
        }

        // println!("found family:\n{:#?}", family);
        Ok(family)
    }

    fn parse_source(&mut self, level: u8, xref: Option<String>) -> Source {
        // skip SOUR tag
        self.tokenizer.next_token();
        let mut source = Source::default();
        source.xref = xref;

        loop {
            if let Token::Level(cur_level) = self.tokenizer.current_token
                && cur_level <= level
            {
                break;
            }
            match &self.tokenizer.current_token {
                Token::Tag(tag) => match tag.as_str() {
                    "DATA" => self.tokenizer.next_token(),
                    "EVEN" => {
                        let events_recorded = self.take_line_value();
                        let mut event = self.parse_event("OTHER", level + 2);
                        event.etype = EventType::SourceData(events_recorded);
                        source.data.events.push(event);
                    }
                    "AGNC" => source.data.agency = Some(self.take_line_value()),
                    "ABBR" => source.abbreviation = Some(self.take_continued_text(level + 1)),
                    "TITL" => source.title = Some(self.take_continued_text(level + 1)),
                    "REPO" => source
                        .repo_citations
                        .push(self.parse_repo_citation(level + 1)),
                    _ => panic!("{} Unhandled Source Tag: {}", self.dbg(), tag),
                },
                Token::Level(_) => self.tokenizer.next_token(),
                _ => panic!("Unhandled Source Token: {:?}", self.tokenizer.current_token),
            }
        }

        // println!("found source:\n{:#?}", source);
        source
    }

    /// Parses REPO top-level tag.
    fn parse_repository(&mut self, level: u8, xref: Option<String>) -> Repository {
        // skip REPO tag
        self.tokenizer.next_token();
        let mut repo = Repository {
            xref,
            name: None,
            address: None,
        };
        loop {
            if let Token::Level(cur_level) = self.tokenizer.current_token
                && cur_level <= level
            {
                break;
            }
            match &self.tokenizer.current_token {
                Token::Tag(tag) => match tag.as_str() {
                    "NAME" => repo.name = Some(self.take_line_value()),
                    "ADDR" => repo.address = Some(self.parse_address(level + 1)),
                    _ => panic!("{} Unhandled Repository Tag: {}", self.dbg(), tag),
                },
                Token::Level(_) => self.tokenizer.next_token(),
                _ => panic!(
                    "Unhandled Repository Token: {:?}",
                    self.tokenizer.current_token
                ),
            }
        }
        // println!("found repository:\n{:#?}", repo);
        repo
    }

    fn parse_custom_tag(&mut self, tag: String) -> CustomData {
        let value = self.take_line_value();
        CustomData { tag, value }
    }

    /// Handle parsing GEDC tag
    fn parse_gedcom_data(&mut self, mut header: Header) -> Header {
        // skip GEDC tag
        self.tokenizer.next_token();

        while self.tokenizer.current_token != Token::Level(1) {
            match &self.tokenizer.current_token {
                Token::Tag(tag) => match tag.as_str() {
                    "VERS" => header.gedcom_version = Some(self.take_line_value()),
                    // this is the only value that makes sense. warn them otherwise.
                    "FORM" => {
                        let form = self.take_line_value();
                        if &form.to_uppercase() != "LINEAGE-LINKED" {
                            println!(
                                "WARNING: Unrecognized GEDCOM form. Expected LINEAGE-LINKED, found {form}"
                            );
                        }
                    }
                    _ => panic!("{} Unhandled GEDC Tag: {}", self.dbg(), tag),
                },
                Token::Level(_) => self.tokenizer.next_token(),
                _ => panic!(
                    "{} Unexpected GEDC Token: {:?}",
                    self.dbg(),
                    &self.tokenizer.current_token
                ),
            }
        }
        header
    }

    fn parse_family_link(&mut self, tag: &str, level: u8) -> FamilyLink {
        let xref = self.take_line_value();
        let mut family_link = FamilyLink::default();
        family_link.xref = xref;
        let link = match tag {
            "FAMC" => FamilyLinkType::Child,
            "FAMS" => FamilyLinkType::Spouse,
            _ => panic!("Unrecognized family type tag: {tag}"),
        };

        family_link.link = link;

        loop {
            if let Token::Level(cur_level) = self.tokenizer.current_token
                && cur_level <= level
            {
                break;
            }
            match &self.tokenizer.current_token {
                Token::Tag(tag) => match tag.as_str() {
                    "PEDI" => {
                        family_link.pedigree =
                            Some(Pedigree::try_from(self.take_line_value().as_str()).unwrap());
                    }
                    _ => panic!("{} Unhandled FamilyLink Tag: {}", self.dbg(), tag),
                },
                Token::Level(_) => self.tokenizer.next_token(),
                _ => panic!(
                    "Unhandled FamilyLink Token: {:?}",
                    self.tokenizer.current_token
                ),
            }
        }

        family_link
    }

    fn parse_repo_citation(&mut self, level: u8) -> RepoCitation {
        let xref = self.take_line_value();
        let mut citation = RepoCitation {
            xref,
            call_number: None,
        };
        loop {
            if let Token::Level(cur_level) = self.tokenizer.current_token
                && cur_level <= level
            {
                break;
            }
            match &self.tokenizer.current_token {
                Token::Tag(tag) => match tag.as_str() {
                    "CALN" => citation.call_number = Some(self.take_line_value()),
                    _ => panic!("{} Unhandled RepoCitation Tag: {}", self.dbg(), tag),
                },
                Token::Level(_) => self.tokenizer.next_token(),
                _ => panic!(
                    "Unhandled RepoCitation Token: {:?}",
                    self.tokenizer.current_token
                ),
            }
        }
        citation
    }

    fn parse_gender(&mut self) -> Gender {
        self.tokenizer.next_token();
        let gender: Gender;
        if let Token::LineValue(gender_string) = &self.tokenizer.current_token {
            gender = match gender_string.as_str() {
                "M" => Gender::Male,
                "F" => Gender::Female,
                "N" => Gender::Nonbinary,
                "U" => Gender::Unknown,
                _ => panic!("{} Unknown gender value {}", self.dbg(), gender_string),
            };
        } else {
            panic!(
                "Expected gender LineValue, found {:?}",
                self.tokenizer.current_token
            );
        }
        self.tokenizer.next_token();
        gender
    }

    fn parse_name(&mut self, level: u8) -> Name {
        let mut name = Name::default();
        name.value = Some(self.take_line_value());

        loop {
            if let Token::Level(cur_level) = self.tokenizer.current_token
                && cur_level <= level
            {
                break;
            }
            match &self.tokenizer.current_token {
                Token::Tag(tag) => match tag.as_str() {
                    "GIVN" => name.given = Some(self.take_line_value()),
                    "NPFX" => name.prefix = Some(self.take_line_value()),
                    "NSFX" => name.suffix = Some(self.take_line_value()),
                    "SPFX" => name.surname_prefix = Some(self.take_line_value()),
                    "SURN" => name.surname = Some(self.take_line_value()),
                    _ => panic!("{} Unhandled Name Tag: {}", self.dbg(), tag),
                },
                Token::Level(_) => self.tokenizer.next_token(),
                _ => panic!("Unhandled Name Token: {:?}", self.tokenizer.current_token),
            }
        }

        name
    }

    fn parse_event(&mut self, tag: &str, level: u8) -> Event {
        self.tokenizer.next_token();
        let mut event = Event::try_from(tag).unwrap();
        loop {
            if let Token::Level(cur_level) = self.tokenizer.current_token
                && cur_level <= level
            {
                break;
            }
            match &self.tokenizer.current_token {
                Token::Tag(tag) => match tag.as_str() {
                    "DATE" => event.date = Some(self.take_line_value()),
                    "PLAC" => event.place = Some(self.take_line_value()),
                    "SOUR" => event.citations.push(self.parse_citation(level + 1)),
                    _ => panic!("{} Unhandled Event Tag: {}", self.dbg(), tag),
                },
                Token::Level(_) => self.tokenizer.next_token(),
                _ => panic!("Unhandled Event Token: {:?}", self.tokenizer.current_token),
            }
        }
        event
    }

    /// Parses ADDR tag
    fn parse_address(&mut self, level: u8) -> Address {
        // skip ADDR tag
        self.tokenizer.next_token();
        let mut address = Address::default();
        let mut value = String::new();

        // handle value on ADDR line
        if let Token::LineValue(addr) = &self.tokenizer.current_token {
            value.push_str(addr);
            self.tokenizer.next_token();
        }

        loop {
            if let Token::Level(cur_level) = self.tokenizer.current_token
                && cur_level <= level
            {
                break;
            }
            match &self.tokenizer.current_token {
                Token::Tag(tag) => match tag.as_str() {
                    "CONT" => {
                        value.push('\n');
                        value.push_str(&self.take_line_value());
                    }
                    "ADR1" => address.adr1 = Some(self.take_line_value()),
                    "ADR2" => address.adr2 = Some(self.take_line_value()),
                    "ADR3" => address.adr3 = Some(self.take_line_value()),
                    "CITY" => address.city = Some(self.take_line_value()),
                    "STAE" => address.state = Some(self.take_line_value()),
                    "POST" => address.post = Some(self.take_line_value()),
                    "CTRY" => address.country = Some(self.take_line_value()),
                    _ => panic!("{} Unhandled Address Tag: {}", self.dbg(), tag),
                },
                Token::Level(_) => self.tokenizer.next_token(),
                _ => panic!(
                    "Unhandled Address Token: {:?}",
                    self.tokenizer.current_token
                ),
            }
        }

        if !value.is_empty() {
            address.value = Some(value);
        }

        address
    }

    fn parse_citation(&mut self, level: u8) -> SourceCitation {
        let mut citation = SourceCitation {
            xref: self.take_line_value(),
            page: None,
        };
        loop {
            if let Token::Level(cur_level) = self.tokenizer.current_token
                && cur_level <= level
            {
                break;
            }
            match &self.tokenizer.current_token {
                Token::Tag(tag) => match tag.as_str() {
                    "PAGE" => citation.page = Some(self.take_line_value()),
                    _ => panic!("{} Unhandled Citation Tag: {}", self.dbg(), tag),
                },
                Token::Level(_) => self.tokenizer.next_token(),
                _ => panic!(
                    "Unhandled Citation Token: {:?}",
                    self.tokenizer.current_token
                ),
            }
        }
        citation
    }

    /// Takes the value of the current line including handling
    /// multi-line values from CONT & CONC tags.
    fn take_continued_text(&mut self, level: u8) -> String {
        let mut value = self.take_line_value();

        loop {
            if let Token::Level(cur_level) = self.tokenizer.current_token
                && cur_level <= level
            {
                break;
            }
            match &self.tokenizer.current_token {
                Token::Tag(tag) => match tag.as_str() {
                    "CONT" => {
                        value.push('\n');
                        value.push_str(&self.take_line_value());
                    }
                    "CONC" => {
                        value.push(' ');
                        value.push_str(&self.take_line_value());
                    }
                    _ => panic!("{} Unhandled Continuation Tag: {}", self.dbg(), tag),
                },
                Token::Level(_) => self.tokenizer.next_token(),
                _ => panic!(
                    "Unhandled Continuation Token: {:?}",
                    self.tokenizer.current_token
                ),
            }
        }

        value
    }

    /// Grabs and returns to the end of the current line as a String
    fn take_line_value(&mut self) -> String {
        let value: String;
        self.tokenizer.next_token();

        if let Token::LineValue(val) = &self.tokenizer.current_token {
            value = val.clone();
        } else {
            panic!(
                "{} Expected LineValue, found {:?}",
                self.dbg(),
                self.tokenizer.current_token
            );
        }
        self.tokenizer.next_token();
        value
    }

    /// Debug function displaying GEDCOM line number of error message.
    fn dbg(&self) -> String {
        format!("line {}:", self.tokenizer.line)
    }
}
