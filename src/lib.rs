use std::collections::HashMap;

use lambda_runtime::Error;

pub struct ImageMetadata {
    pub portrait_hight: u32,
    pub portrait_width: u32,
    pub x1: u32,
    pub y1: u32,
}

impl ImageMetadata {
    pub fn new(metadata: &HashMap<String, String>) -> Result<Self, Error> {
        let portrait_hight: u32 = metadata
            .get("portraithight".into())
            .ok_or("should have portrait hight")?
            .parse()?;
        let portrait_width: u32 = metadata
            .get("portraitwidth".into())
            .ok_or("should have portrait width")?
            .parse()?;
        let x1: u32 = metadata.get("x1".into()).ok_or("should have x1")?.parse()?;
        let y1: u32 = metadata.get("y1".into()).ok_or("should have y1")?.parse()?;

        Ok(ImageMetadata {
            portrait_hight,
            portrait_width,
            x1,
            y1,
        })
    }
}
