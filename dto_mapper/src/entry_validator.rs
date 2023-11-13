use std::{collections::{HashMap, HashSet}, ops::Add};

use crate::{struct_entry::StructEntry, mapper_entry::MapperEntry};

#[derive(Debug)]
pub enum FieldError{
    DupField(String),
    MissingField(String)
}

#[derive(Debug)]
pub enum ValidationError{
    MapperEntryError(Vec<FieldError>),
    StructEntryError(Vec<FieldError>),
    DtoNameDuplicated(Vec<String>),
    MissingPropertyError(String),
}


pub fn validate_entry_data(st_entry: &StructEntry, mp_entries: Vec<MapperEntry>) -> Result<(), ValidationError>{
    validate_mapper_entries(& mp_entries)?;
    validate_struct_entry(st_entry, & mp_entries)?;
    validate_dto_name(&mp_entries)?;
    validate_map_ignore(&mp_entries)?;
    Ok(())
}

fn validate_map_ignore(mp_entries: &Vec<MapperEntry>) -> Result<(), ValidationError>{
    //There should be at least a map attribute or an ignore attribute per mapper entry
    // valid mapper entry = ignore.len() > 0 || map.len() > 0
    // invalid mapper entry = ignore.len() == 0 and map.len()==0
    let invalid_entries : Vec<String> = mp_entries.iter()
    .filter(| mp_entry| mp_entry.map.len() == 0 && mp_entry.ignore.len() == 0)
    .map(|mp_entry| mp_entry.dto.to_string())
    .collect();

    if invalid_entries.len() > 0{
        return Err(ValidationError::MissingPropertyError(format!("mapper requires a `map` or an `ignore` property")));
    }

    Ok(())
}

fn validate_dto_name(mp_entries: &Vec<MapperEntry>) -> Result<(), ValidationError>{
    let mut dto_hash: HashMap<String,u8>  = HashMap::new();
    mp_entries.iter().for_each(|mp_entry| {
        if let Some((ref key,ref count)) = dto_hash.get_key_value(& mp_entry.dto)
         {
            dto_hash.insert(key.to_string(),count.add(1))
         }
         else{dto_hash.insert(mp_entry.dto.to_string(), 1)};
    });

    let dto_dup: Vec<String> = dto_hash.iter()
    .filter(|(ref _k, & val)| val>1)
    .map(|(ref dto_val, &_)| dto_val.to_string())
    .collect();
    if dto_dup.len() > 0{
        return Err(ValidationError::DtoNameDuplicated(dto_dup));
    }
    Ok(())
}

fn validate_struct_entry(st_entry: &StructEntry, mp_entries: &Vec<MapperEntry>) -> Result<(),ValidationError>{
    //extract a hashset of the fields name from the struct
    let field_set : HashSet<String>= st_entry.field_entries.iter()
    .map(|f| f.field_name.as_str().to_string())
    .collect();

    let mut errors : Vec<FieldError> = Vec::new();
    for ref mp_entry in mp_entries{
        let missing_fields:  Vec<String> = mp_entry.map.iter()
        .filter(|& mp_value| ! field_set.contains(&mp_value.from_field))
        .map(| m| m.from_field.as_str().to_string())
        .collect();
        if missing_fields.len() > 0{
            errors.push(FieldError::MissingField(
                format!("{} field name doesn't exist in structure={}. List of wrong map field names : {:?}"
                ,mp_entry.dto, st_entry.name,missing_fields  )
            ));
        }
        
    }
    //println!("Validation Error : {:?}", errors);
    if errors.len() > 0 {
        return Err(ValidationError::StructEntryError(errors));
    }
    Ok(())
}

fn validate_mapper_entries(mp_entries: &Vec<MapperEntry>) -> Result<(), ValidationError> {
    //verify if we have duplicate field names in mp_entry for source and destination map fields

    let mut errors: Vec<FieldError> = Vec::new();

    for mp_entry in mp_entries{
        let  mut from_set: HashMap<String,u8> = HashMap::new();
        let mut to_set: HashMap<String,u8> = HashMap::new();
        mp_entry.map.iter().for_each(|m_value| {   
            if let Some((ref key, ref count))=  from_set.get_key_value( & m_value.from_field){
                    from_set.insert(key.to_string(), count.add(1));
            }
            else { from_set.insert(m_value.from_field.to_string(), 1); }

            //if from_field is mapped to to_field
            if let Some(ref to_field)= m_value.to_field {
                if let Some((key, count))=  to_set.get_key_value( to_field){
                    to_set.insert(key.to_string(), count.add(1));
                }
                else { to_set.insert(to_field.to_string(), 1); }
            }
        });
        println!();
        println!("dto={} from_field_map={:?}",mp_entry.dto,from_set);
        println!("dto={} to_field_map={:?}",mp_entry.dto,to_set);


        // to_keys.len() will always be lesser than or equal from_keys.len()
        let dup_fields : Vec<String>= to_set.iter().filter(|(ref key,&_c)| from_set.contains_key(& key.to_string()) )
        .map(|(ref key,&_c)| key.to_string())
        .collect();
    
        if dup_fields.len() > 0{
            errors.push(FieldError::DupField( format!("duplicate mapping destination keys found in dto={} entry: {:?}",mp_entry.dto,dup_fields)) );
        }

        let dup_from: Vec<String> = from_set.iter().filter(|(ref _key, & count)| count > 1 )
        .map(|(ref k,&_)| k.to_string()).collect();
        
        if dup_from.len() > 0 {
            errors.push(FieldError::DupField( format!("duplicate source key names found in dto={} entry: {:?}",mp_entry.dto,dup_from)) );
        }


        let dup_to: Vec<String> = to_set.iter().filter(|(ref _key, & c)| c > 1 )
        .map(|(ref k,&_)| k.to_string()).collect();
        
    
        if dup_to.len() > 0 {
            errors.push(FieldError::DupField( format!("duplicate destination key names found in dto={} entry: {:?}",mp_entry.dto,dup_to)) );
        }
    }

    if errors.len() > 0 {
        return Err(ValidationError::MapperEntryError(errors))
    }

    Ok(())
}