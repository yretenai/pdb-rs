use std::collections::BTreeSet;
use std::fmt;

use pdb::FallibleIterator;

type TypeSet = BTreeSet<pdb::TypeIndex>;

pub fn type_name<'p>(
    type_finder: &pdb::TypeFinder<'p>,
    type_index: pdb::TypeIndex,
    needed_types: &mut TypeSet,
) -> pdb::Result<String> {
    let mut name = match type_finder.find(type_index)?.parse()? {
        pdb::TypeData::Primitive(data) => {
            let mut name = match data.kind {
                pdb::PrimitiveKind::Void => "void".to_string(),
                pdb::PrimitiveKind::HRESULT => "HRESULT".to_string(),

                pdb::PrimitiveKind::RChar => "char8_t".to_string(),
                pdb::PrimitiveKind::RChar16 => "char16_t".to_string(),
                pdb::PrimitiveKind::RChar32 => "char32_t".to_string(),
                pdb::PrimitiveKind::WChar => "wchar_t".to_string(),

                pdb::PrimitiveKind::I8 => "int8_t".to_string(),
                pdb::PrimitiveKind::U8 => "uint8_t".to_string(),
                pdb::PrimitiveKind::Char => "char".to_string(),
                pdb::PrimitiveKind::UChar => "unsigned char".to_string(),

                pdb::PrimitiveKind::I16 => "int16_t".to_string(),
                pdb::PrimitiveKind::U16 => "uint16_t".to_string(),
                pdb::PrimitiveKind::Short => "uint16_t".to_string(),
                pdb::PrimitiveKind::UShort => "int16_t".to_string(),

                pdb::PrimitiveKind::I32 => "int32_t".to_string(),
                pdb::PrimitiveKind::U32 => "uint32_t".to_string(),
                pdb::PrimitiveKind::Long => "int32_t".to_string(),
                pdb::PrimitiveKind::ULong => "uint32_t".to_string(),

                pdb::PrimitiveKind::I64 => "int64_t".to_string(),
                pdb::PrimitiveKind::U64 => "uint64_t".to_string(),
                pdb::PrimitiveKind::Quad => "int64_t".to_string(),
                pdb::PrimitiveKind::UQuad => "uint64_t".to_string(),

                pdb::PrimitiveKind::I128 => "int128_t".to_string(),
                pdb::PrimitiveKind::U128 => "uint128_t".to_string(),
                pdb::PrimitiveKind::Octa => "int128_t".to_string(),
                pdb::PrimitiveKind::UOcta => "uint128_t".to_string(),

                pdb::PrimitiveKind::F16 => "half".to_string(),
                pdb::PrimitiveKind::F32 => "float".to_string(),
                pdb::PrimitiveKind::F32PP => "binary32".to_string(),
                pdb::PrimitiveKind::F48 => "binary48".to_string(),
                pdb::PrimitiveKind::F64 => "double".to_string(),
                pdb::PrimitiveKind::F80 => "binary80".to_string(),
                pdb::PrimitiveKind::F128 => "binary128".to_string(),

                pdb::PrimitiveKind::Bool8 => "bool".to_string(),
                pdb::PrimitiveKind::Bool16 => "bool16".to_string(),
                pdb::PrimitiveKind::Bool32 => "bool32".to_string(),
                pdb::PrimitiveKind::Bool64 => "bool64".to_string(),

                _ => format!("unhandled_primitive.kind /* {:?} */", data.kind),
            };

            if data.indirection.is_some() {
                name.push_str(" *");
            }

            name
        }

        pdb::TypeData::Class(data) => {
            needed_types.insert(type_index);
            data.name.to_string().into_owned()
        }

        pdb::TypeData::Enumeration(data) => {
            needed_types.insert(type_index);
            data.name.to_string().into_owned()
        }

        pdb::TypeData::Union(data) => {
            needed_types.insert(type_index);
            data.name.to_string().into_owned()
        }

        pdb::TypeData::Pointer(data) => format!(
            "{}*",
            type_name(type_finder, data.underlying_type, needed_types)?
        ),

        pdb::TypeData::Modifier(data) => {
            if data.constant {
                format!(
                    "const {}",
                    type_name(type_finder, data.underlying_type, needed_types)?
                )
            } else if data.volatile {
                format!(
                    "volatile {}",
                    type_name(type_finder, data.underlying_type, needed_types)?
                )
            } else {
                // ?
                type_name(type_finder, data.underlying_type, needed_types)?
            }
        }

        pdb::TypeData::Array(data) => {
            let mut name = type_name(type_finder, data.element_type, needed_types)?;
            for size in data.dimensions {
                name = format!("{}[{}]", name, size);
            }
            name
        }

        _ => format!("Type{} /* TODO: figure out how to name it */", type_index),
    };

    // TODO: search and replace std:: patterns
    if name == "std::basic_string<char,std::char_traits<char>,std::allocator<char> >" {
        name = "std::string".to_string();
    }

    Ok(name)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Class<'p> {
    kind: pdb::ClassKind,
    name: pdb::RawString<'p>,
    base_classes: Vec<BaseClass>,
    fields: Vec<Field<'p>>,
    instance_methods: Vec<Method<'p>>,
    static_methods: Vec<Method<'p>>,
}

impl<'p> Class<'p> {
    #[allow(clippy::unnecessary_wraps)]
    fn add_derived_from(
        &mut self,
        _: &pdb::TypeFinder<'p>,
        _: pdb::TypeIndex,
        _: &mut TypeSet,
    ) -> pdb::Result<()> {
        // TODO
        Ok(())
    }

    fn add_fields(
        &mut self,
        type_finder: &pdb::TypeFinder<'p>,
        type_index: pdb::TypeIndex,
        needed_types: &mut TypeSet,
    ) -> pdb::Result<()> {
        match type_finder.find(type_index)?.parse()? {
            pdb::TypeData::FieldList(data) => {
                for field in &data.fields {
                    self.add_field(type_finder, field, needed_types)?;
                }

                if let Some(continuation) = data.continuation {
                    // recurse
                    self.add_fields(type_finder, continuation, needed_types)?;
                }
            }
            other => {
                println!(
                    "trying to Class::add_fields() got {} -> {:?}",
                    type_index, other
                );
                panic!("unexpected type in Class::add_fields()");
            }
        }

        Ok(())
    }

    fn add_field(
        &mut self,
        type_finder: &pdb::TypeFinder<'p>,
        field: &pdb::TypeData<'p>,
        needed_types: &mut TypeSet,
    ) -> pdb::Result<()> {
        match *field {
            pdb::TypeData::Member(ref data) => {
                // TODO: attributes (static, virtual, etc.)
                self.fields.push(Field {
                    type_name: type_name(type_finder, data.field_type, needed_types)?,
                    name: data.name,
                    offset: data.offset,
                });
            }

            pdb::TypeData::Method(ref data) => {
                let method = Method::find(
                    data.name,
                    data.attributes,
                    type_finder,
                    data.method_type,
                    needed_types,
                )?;
                if data.attributes.is_static() {
                    self.static_methods.push(method);
                } else {
                    self.instance_methods.push(method);
                }
            }

            pdb::TypeData::OverloadedMethod(ref data) => {
                // this just means we have more than one method with the same name
                // find the method list
                match type_finder.find(data.method_list)?.parse()? {
                    pdb::TypeData::MethodList(method_list) => {
                        for pdb::MethodListEntry {
                            attributes,
                            method_type,
                            ..
                        } in method_list.methods
                        {
                            // hooray
                            let method = Method::find(
                                data.name,
                                attributes,
                                type_finder,
                                method_type,
                                needed_types,
                            )?;

                            if attributes.is_static() {
                                self.static_methods.push(method);
                            } else {
                                self.instance_methods.push(method);
                            }
                        }
                    }
                    other => {
                        println!(
                            "processing OverloadedMethod, expected MethodList, got {} -> {:?}",
                            data.method_list, other
                        );
                        panic!("unexpected type in Class::add_field()");
                    }
                }
            }

            pdb::TypeData::BaseClass(ref data) => self.base_classes.push(BaseClass {
                type_name: type_name(type_finder, data.base_class, needed_types)?,
                offset: data.offset,
            }),

            pdb::TypeData::VirtualBaseClass(ref data) => self.base_classes.push(BaseClass {
                type_name: type_name(type_finder, data.base_class, needed_types)?,
                offset: data.base_pointer_offset,
            }),

            _ => {
                // ignore everything else even though that's sad
            }
        }

        Ok(())
    }
}

impl fmt::Display for Class<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} ",
            match self.kind {
                pdb::ClassKind::Class => "class",
                pdb::ClassKind::Struct => "struct",
                pdb::ClassKind::Interface => "interface", // when can this happen?
            },
            self.name.to_string()
        )?;

        if !self.base_classes.is_empty() {
            for (i, base) in self.base_classes.iter().enumerate() {
                let prefix = match i {
                    0 => ":",
                    _ => ",",
                };
                write!(f, "{} {}", prefix, base.type_name)?;
            }
        }

        writeln!(f, " {{")?;

        for base in &self.base_classes {
            writeln!(
                f,
                "\t/* offset 0x{:04x} */ /* fields for {} */",
                base.offset, base.type_name
            )?;
        }

        for field in &self.fields {
            writeln!(
                f,
                "\t/* offset 0x{:04x} */ {} {};",
                field.offset,
                field.type_name,
                field.name.to_string()
            )?;
        }

        if !self.instance_methods.is_empty() {
            writeln!(f, "\t")?;
            for method in &self.instance_methods {
                writeln!(
                    f,
                    "\t{}{} {}({});",
                    if method.is_virtual { "virtual " } else { "" },
                    method.return_type_name,
                    method.name.to_string(),
                    method.arguments.join(", ")
                )?;
            }
        }

        if !self.static_methods.is_empty() {
            writeln!(f, "\t")?;
            for method in &self.static_methods {
                writeln!(
                    f,
                    "\t{}static {} {}({});",
                    if method.is_virtual { "virtual " } else { "" },
                    method.return_type_name,
                    method.name.to_string(),
                    method.arguments.join(", ")
                )?;
            }
        }

        writeln!(f, "}}")?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BaseClass {
    type_name: String,
    offset: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Field<'p> {
    type_name: String,
    name: pdb::RawString<'p>,
    offset: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Method<'p> {
    name: pdb::RawString<'p>,
    return_type_name: String,
    arguments: Vec<String>,
    is_virtual: bool,
}

impl<'p> Method<'p> {
    fn find(
        name: pdb::RawString<'p>,
        attributes: pdb::FieldAttributes,
        type_finder: &pdb::TypeFinder<'p>,
        type_index: pdb::TypeIndex,
        needed_types: &mut TypeSet,
    ) -> pdb::Result<Method<'p>> {
        match type_finder.find(type_index)?.parse()? {
            pdb::TypeData::MemberFunction(data) => Ok(Method {
                name,
                return_type_name: type_name(type_finder, data.return_type, needed_types)?,
                arguments: argument_list(type_finder, data.argument_list, needed_types)?,
                is_virtual: attributes.is_virtual(),
            }),

            other => {
                println!("other: {:?}", other);
                Err(pdb::Error::UnimplementedFeature("that"))
            }
        }
    }
}

fn argument_list<'p>(
    type_finder: &pdb::TypeFinder<'p>,
    type_index: pdb::TypeIndex,
    needed_types: &mut TypeSet,
) -> pdb::Result<Vec<String>> {
    match type_finder.find(type_index)?.parse()? {
        pdb::TypeData::ArgumentList(data) => {
            let mut args: Vec<String> = Vec::new();
            for arg_type in data.arguments {
                args.push(type_name(type_finder, arg_type, needed_types)?);
            }
            Ok(args)
        }
        _ => Err(pdb::Error::UnimplementedFeature(
            "argument list of non-argument-list type",
        )),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Enum<'p> {
    name: pdb::RawString<'p>,
    underlying_type_name: String,
    values: Vec<EnumValue<'p>>,
}

impl<'p> Enum<'p> {
    fn add_fields(
        &mut self,
        type_finder: &pdb::TypeFinder<'p>,
        type_index: pdb::TypeIndex,
        needed_types: &mut TypeSet,
    ) -> pdb::Result<()> {
        match type_finder.find(type_index)?.parse()? {
            pdb::TypeData::FieldList(data) => {
                for field in &data.fields {
                    self.add_field(type_finder, field, needed_types);
                }

                if let Some(continuation) = data.continuation {
                    // recurse
                    self.add_fields(type_finder, continuation, needed_types)?;
                }
            }
            other => {
                println!(
                    "trying to Enum::add_fields() got {} -> {:?}",
                    type_index, other
                );
                panic!("unexpected type in Enum::add_fields()");
            }
        }

        Ok(())
    }

    fn add_field(&mut self, _: &pdb::TypeFinder<'p>, field: &pdb::TypeData<'p>, _: &mut TypeSet) {
        // ignore everything else even though that's sad
        if let pdb::TypeData::Enumerate(ref data) = field {
            self.values.push(EnumValue {
                name: data.name,
                value: data.value,
            });
        }
    }
}

impl fmt::Display for Enum<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "enum {} : {} {{",
            self.name.to_string(),
            self.underlying_type_name
        )?;

        for value in &self.values {
            writeln!(
                f,
                "\t{} = {},",
                value.name.to_string(),
                match value.value {
                    pdb::Variant::U8(v) => format!("0x{:02x}", v),
                    pdb::Variant::U16(v) => format!("0x{:04x}", v),
                    pdb::Variant::U32(v) => format!("0x{:08x}", v),
                    pdb::Variant::U64(v) => format!("0x{:016x}", v),
                    pdb::Variant::I8(v) => format!("{}", v),
                    pdb::Variant::I16(v) => format!("{}", v),
                    pdb::Variant::I32(v) => format!("{}", v),
                    pdb::Variant::I64(v) => format!("{}", v),
                }
            )?;
        }
        writeln!(f, "}}")?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EnumValue<'p> {
    name: pdb::RawString<'p>,
    value: pdb::Variant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ForwardReference<'p> {
    kind: pdb::ClassKind,
    name: pdb::RawString<'p>,
}

impl fmt::Display for ForwardReference<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{} {};",
            match self.kind {
                pdb::ClassKind::Class => "class",
                pdb::ClassKind::Struct => "struct",
                pdb::ClassKind::Interface => "interface", // when can this happen?
            },
            self.name.to_string()
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Data<'p> {
    forward_references: Vec<ForwardReference<'p>>,
    classes: Vec<Class<'p>>,
    enums: Vec<Enum<'p>>,
}

impl fmt::Display for Data<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.forward_references.is_empty() {
            writeln!(f)?;
            for e in &self.forward_references {
                e.fmt(f)?;
            }
        }

        for e in &self.enums {
            writeln!(f)?;
            e.fmt(f)?;
        }

        for class in &self.classes {
            writeln!(f)?;
            class.fmt(f)?;
        }

        Ok(())
    }
}

impl<'p> Data<'p> {
    fn new() -> Data<'p> {
        Data {
            forward_references: Vec::new(),
            classes: Vec::new(),
            enums: Vec::new(),
        }
    }

    fn add(
        &mut self,
        type_finder: &pdb::TypeFinder<'p>,
        type_index: pdb::TypeIndex,
        needed_types: &mut TypeSet,
    ) -> pdb::Result<()> {
        match type_finder.find(type_index)?.parse()? {
            pdb::TypeData::Class(data) => {
                if data.properties.forward_reference() {
                    self.forward_references.push(ForwardReference {
                        kind: data.kind,
                        name: data.name,
                    });

                    return Ok(());
                }

                let mut class = Class {
                    kind: data.kind,
                    name: data.name,
                    fields: Vec::new(),
                    base_classes: Vec::new(),
                    instance_methods: Vec::new(),
                    static_methods: Vec::new(),
                };

                if let Some(derived_from) = data.derived_from {
                    class.add_derived_from(type_finder, derived_from, needed_types)?;
                }

                if let Some(fields) = data.fields {
                    class.add_fields(type_finder, fields, needed_types)?;
                }

                self.classes.insert(0, class);
            }

            pdb::TypeData::Enumeration(data) => {
                let mut e = Enum {
                    name: data.name,
                    underlying_type_name: type_name(
                        type_finder,
                        data.underlying_type,
                        needed_types,
                    )?,
                    values: Vec::new(),
                };

                e.add_fields(type_finder, data.fields, needed_types)?;

                self.enums.insert(0, e);
            }

            // ignore
            other => eprintln!("warning: don't know how to add {:?}", other),
        }

        Ok(())
    }
}

fn write_type(filename: &str, type_name: &str) -> pdb::Result<()> {
    let file = std::fs::File::open(filename)?;
    let mut pdb = pdb::PDB::open(file)?;

    let type_information = pdb.type_information()?;
    let mut type_finder = type_information.finder();

    let mut needed_types = TypeSet::new();
    let mut data = Data::new();

    let mut type_iter = type_information.iter();
    while let Some(typ) = type_iter.next()? {
        // keep building the index
        type_finder.update(&type_iter);

        if let Ok(pdb::TypeData::Class(class)) = typ.parse() {
            if class.name.as_bytes() == type_name.as_bytes()
                && !class.properties.forward_reference()
            {
                data.add(&type_finder, typ.index(), &mut needed_types)?;
                break;
            }
        } else if let Ok(pdb::TypeData::Union(struct_union)) = typ.parse() {
            if struct_union.name.as_bytes() == type_name.as_bytes()
                && !struct_union.properties.forward_reference()
            {
                data.add(&type_finder, typ.index(), &mut needed_types)?;
                break;
            }
        } else if let Ok(pdb::TypeData::Enumeration(enumeration)) = typ.parse() {
            if enumeration.name.as_bytes() == type_name.as_bytes()
                && !enumeration.properties.forward_reference()
            {
                data.add(&type_finder, typ.index(), &mut needed_types)?;
                break;
            }
        }
    }

    // add all the needed types iteratively until we're done
    while let Some(type_index) = needed_types.iter().next_back().copied() {
        // remove it
        needed_types.remove(&type_index);

        // add the type
        data.add(&type_finder, type_index, &mut needed_types)?;
    }

    if data.classes.is_empty() && data.enums.is_empty() {
        eprintln!("sorry, type {} was not found", type_name);
    } else {
        println!("{}", data);
    }

    Ok(())
}

fn print_usage(program: &str, opts: getopts::Options) {
    let brief = format!("Usage: {} input.pdb TypeName", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let program = args[0].clone();

    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!("{}", f.to_string()),
    };

    let (filename, type_names) = if matches.free.len() > 1 {
        (&matches.free[0], &matches.free[1..])
    } else {
        print_usage(&program, opts);
        return;
    };

    for type_name in type_names {
        match write_type(filename, type_name) {
            Ok(_) => (),
            Err(e) => eprintln!("error dumping PDB: {}", e),
        }
    }
}
