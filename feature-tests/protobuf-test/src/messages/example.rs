// This file is generated by rust-protobuf 2.23.0. Do not edit
// @generated

// https://github.com/rust-lang/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy::all)]

#![allow(unused_attributes)]
#![cfg_attr(rustfmt, rustfmt::skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unused_imports)]
#![allow(unused_results)]
//! Generated file from `example.proto`

/// Generated files are compatible only with the same version
/// of protobuf runtime.
// const _PROTOBUF_VERSION_CHECK: () = ::protobuf::VERSION_2_23_0;

#[derive(PartialEq,Clone,Default)]
pub struct SearchRequest {
    // message fields
    pub query: ::std::string::String,
    pub page_number: i32,
    pub result_per_page: i32,
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::CachedSize,
}

impl<'a> ::std::default::Default for &'a SearchRequest {
    fn default() -> &'a SearchRequest {
        <SearchRequest as ::protobuf::Message>::default_instance()
    }
}

impl SearchRequest {
    pub fn new() -> SearchRequest {
        ::std::default::Default::default()
    }

    // string query = 1;


    pub fn get_query(&self) -> &str {
        &self.query
    }
    pub fn clear_query(&mut self) {
        self.query.clear();
    }

    // Param is passed by value, moved
    pub fn set_query(&mut self, v: ::std::string::String) {
        self.query = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_query(&mut self) -> &mut ::std::string::String {
        &mut self.query
    }

    // Take field
    pub fn take_query(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.query, ::std::string::String::new())
    }

    // int32 page_number = 2;


    pub fn get_page_number(&self) -> i32 {
        self.page_number
    }
    pub fn clear_page_number(&mut self) {
        self.page_number = 0;
    }

    // Param is passed by value, moved
    pub fn set_page_number(&mut self, v: i32) {
        self.page_number = v;
    }

    // int32 result_per_page = 3;


    pub fn get_result_per_page(&self) -> i32 {
        self.result_per_page
    }
    pub fn clear_result_per_page(&mut self) {
        self.result_per_page = 0;
    }

    // Param is passed by value, moved
    pub fn set_result_per_page(&mut self, v: i32) {
        self.result_per_page = v;
    }
}

impl ::protobuf::Message for SearchRequest {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.query)?;
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_int32()?;
                    self.page_number = tmp;
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_int32()?;
                    self.result_per_page = tmp;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if !self.query.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.query);
        }
        if self.page_number != 0 {
            my_size += ::protobuf::rt::value_size(2, self.page_number, ::protobuf::wire_format::WireTypeVarint);
        }
        if self.result_per_page != 0 {
            my_size += ::protobuf::rt::value_size(3, self.result_per_page, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        if !self.query.is_empty() {
            os.write_string(1, &self.query)?;
        }
        if self.page_number != 0 {
            os.write_int32(2, self.page_number)?;
        }
        if self.result_per_page != 0 {
            os.write_int32(3, self.result_per_page)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &dyn (::std::any::Any) {
        self as &dyn (::std::any::Any)
    }
    fn as_any_mut(&mut self) -> &mut dyn (::std::any::Any) {
        self as &mut dyn (::std::any::Any)
    }
    fn into_any(self: ::std::boxed::Box<Self>) -> ::std::boxed::Box<dyn (::std::any::Any)> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        Self::descriptor_static()
    }

    fn new() -> SearchRequest {
        SearchRequest::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static descriptor: ::protobuf::rt::LazyV2<::protobuf::reflect::MessageDescriptor> = ::protobuf::rt::LazyV2::INIT;
        descriptor.get(|| {
            let mut fields = ::std::vec::Vec::new();
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                "query",
                |m: &SearchRequest| { &m.query },
                |m: &mut SearchRequest| { &mut m.query },
            ));
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeInt32>(
                "page_number",
                |m: &SearchRequest| { &m.page_number },
                |m: &mut SearchRequest| { &mut m.page_number },
            ));
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeInt32>(
                "result_per_page",
                |m: &SearchRequest| { &m.result_per_page },
                |m: &mut SearchRequest| { &mut m.result_per_page },
            ));
            ::protobuf::reflect::MessageDescriptor::new_pb_name::<SearchRequest>(
                "SearchRequest",
                fields,
                file_descriptor_proto()
            )
        })
    }

    fn default_instance() -> &'static SearchRequest {
        static instance: ::protobuf::rt::LazyV2<SearchRequest> = ::protobuf::rt::LazyV2::INIT;
        instance.get(SearchRequest::new)
    }
}

impl ::protobuf::Clear for SearchRequest {
    fn clear(&mut self) {
        self.query.clear();
        self.page_number = 0;
        self.result_per_page = 0;
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for SearchRequest {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for SearchRequest {
    fn as_ref(&self) -> ::protobuf::reflect::ReflectValueRef {
        ::protobuf::reflect::ReflectValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct SearchResponse {
    // message fields
    pub results: ::protobuf::RepeatedField<::std::string::String>,
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::CachedSize,
}

impl<'a> ::std::default::Default for &'a SearchResponse {
    fn default() -> &'a SearchResponse {
        <SearchResponse as ::protobuf::Message>::default_instance()
    }
}

impl SearchResponse {
    pub fn new() -> SearchResponse {
        ::std::default::Default::default()
    }

    // repeated string results = 1;


    pub fn get_results(&self) -> &[::std::string::String] {
        &self.results
    }
    pub fn clear_results(&mut self) {
        self.results.clear();
    }

    // Param is passed by value, moved
    pub fn set_results(&mut self, v: ::protobuf::RepeatedField<::std::string::String>) {
        self.results = v;
    }

    // Mutable pointer to the field.
    pub fn mut_results(&mut self) -> &mut ::protobuf::RepeatedField<::std::string::String> {
        &mut self.results
    }

    // Take field
    pub fn take_results(&mut self) -> ::protobuf::RepeatedField<::std::string::String> {
        ::std::mem::replace(&mut self.results, ::protobuf::RepeatedField::new())
    }
}

impl ::protobuf::Message for SearchResponse {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_repeated_string_into(wire_type, is, &mut self.results)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        for value in &self.results {
            my_size += ::protobuf::rt::string_size(1, &value);
        };
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        for v in &self.results {
            os.write_string(1, &v)?;
        };
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &dyn (::std::any::Any) {
        self as &dyn (::std::any::Any)
    }
    fn as_any_mut(&mut self) -> &mut dyn (::std::any::Any) {
        self as &mut dyn (::std::any::Any)
    }
    fn into_any(self: ::std::boxed::Box<Self>) -> ::std::boxed::Box<dyn (::std::any::Any)> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        Self::descriptor_static()
    }

    fn new() -> SearchResponse {
        SearchResponse::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static descriptor: ::protobuf::rt::LazyV2<::protobuf::reflect::MessageDescriptor> = ::protobuf::rt::LazyV2::INIT;
        descriptor.get(|| {
            let mut fields = ::std::vec::Vec::new();
            fields.push(::protobuf::reflect::accessor::make_repeated_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                "results",
                |m: &SearchResponse| { &m.results },
                |m: &mut SearchResponse| { &mut m.results },
            ));
            ::protobuf::reflect::MessageDescriptor::new_pb_name::<SearchResponse>(
                "SearchResponse",
                fields,
                file_descriptor_proto()
            )
        })
    }

    fn default_instance() -> &'static SearchResponse {
        static instance: ::protobuf::rt::LazyV2<SearchResponse> = ::protobuf::rt::LazyV2::INIT;
        instance.get(SearchResponse::new)
    }
}

impl ::protobuf::Clear for SearchResponse {
    fn clear(&mut self) {
        self.results.clear();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for SearchResponse {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for SearchResponse {
    fn as_ref(&self) -> ::protobuf::reflect::ReflectValueRef {
        ::protobuf::reflect::ReflectValueRef::Message(self)
    }
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\rexample.proto\"n\n\rSearchRequest\x12\x14\n\x05query\x18\x01\x20\x01\
    (\tR\x05query\x12\x1f\n\x0bpage_number\x18\x02\x20\x01(\x05R\npageNumber\
    \x12&\n\x0fresult_per_page\x18\x03\x20\x01(\x05R\rresultPerPage\"*\n\x0e\
    SearchResponse\x12\x18\n\x07results\x18\x01\x20\x03(\tR\x07resultsb\x06p\
    roto3\
";

static file_descriptor_proto_lazy: ::protobuf::rt::LazyV2<::protobuf::descriptor::FileDescriptorProto> = ::protobuf::rt::LazyV2::INIT;

fn parse_descriptor_proto() -> ::protobuf::descriptor::FileDescriptorProto {
    ::protobuf::Message::parse_from_bytes(file_descriptor_proto_data).unwrap()
}

pub fn file_descriptor_proto() -> &'static ::protobuf::descriptor::FileDescriptorProto {
    file_descriptor_proto_lazy.get(|| {
        parse_descriptor_proto()
    })
}
