use quick_xml::events::Event;
use quick_xml::name::QName;
use quick_xml::Reader;

use quick_xml::events::attributes::Attribute;
use xfbin::nucc_chunk::NuccChunkType;
use xfbin::nucc::{NuccBinary, NuccStruct, NuccStructInfo};
use xfbin::xfbin::XfbinPage;
use xfbin::Xfbin;

#[derive(Debug)]
pub struct Frame {
    no: String,
    p_gp: Vec<Pgp>,
    p_glare: Vec<PGlare>,
    p_softfocus: Vec<PSoftFocus>,
    p_dof: Vec<PDof>,

}

#[derive(Debug, Clone)]
struct Pgp {
    name: String,
    value: String,
}

#[derive(Debug)]
struct PGlare {
    threshold: String,
    subtraction_color: String,
    composition_intensity: String,
}

#[derive(Debug)]
struct PDof {
    focus_distance: String,
    near_distance: String,
    far_distance: String,
    blur_max_far: String,
    blur_edge: String
}

#[derive(Debug)]
struct PSoftFocus {
    intensity: String,
}

pub fn get_frame_settings(xml: &str) -> Vec<Frame> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);
    let mut buf = Vec::new();

    let mut inside_frame = false;
    let mut inside_setting = false;
    let mut inside_param = false;
    let mut frames = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name() == QName(b"frame") => {
                inside_frame = true;
                frames.push(Frame {
                    no: String::new(),
                    p_gp: Vec::new(),
                    p_glare: Vec::new(),
                    p_softfocus: Vec::new(),
                    p_dof: Vec::new(),
                });

                for attr in e.attributes() {
                    let Attribute { key, value } = attr.unwrap();
                    if key == QName(b"no") {
                        frames.last_mut().unwrap().no = String::from_utf8_lossy(&value).to_string();
                    }
                }
            }
            Ok(Event::Start(ref e)) if inside_frame && e.name() == QName(b"setting") => {
                inside_setting = true;
            }
            Ok(Event::Start(ref e)) if inside_setting && e.name() == QName(b"param") => {
                inside_param = true;
            }
            Ok(Event::End(ref e)) if e.name() == QName(b"param") => {
                inside_param = false;
            }
            Ok(Event::End(ref e)) if e.name() == QName(b"setting") => {
                inside_setting = false;
            }
            Ok(Event::End(ref e)) if e.name() == QName(b"frame") => {
                inside_frame = false;
            }
            Ok(Event::Empty(ref e)) if inside_param && e.name() == QName(b"p_gp") => {
                // Extract attributes
                let mut p_gp = Pgp {
                    name: String::new(),
                    value: String::new(),
                };
                for attr in e.attributes() {
                    let Attribute { key, value } = attr.unwrap();
                    if key == QName(b"name") {
                        p_gp.name = String::from_utf8_lossy(&value).to_string();
                    } else if key == QName(b"value") {
                        p_gp.value = String::from_utf8_lossy(&value).to_string();
                    }
                }
                frames.last_mut().unwrap().p_gp.push(p_gp);
            }
            Ok(Event::Empty(ref e)) if inside_param && e.name() == QName(b"p_glare") => {
                // Extract attributes
                let mut p_glare = PGlare {
                    threshold: String::new(),
                    subtraction_color: String::new(),
                    composition_intensity: String::new(),
                };
                for attr in e.attributes() {
                    let Attribute { key, value } = attr.unwrap();
                    if key == QName(b"threshold") {
                        p_glare.threshold = String::from_utf8_lossy(&value).to_string();
                    } else if key == QName(b"subtractionColor") {
                        p_glare.subtraction_color = String::from_utf8_lossy(&value).to_string();
                    } else if key == QName(b"compositionIntensity") {
                        p_glare.composition_intensity = String::from_utf8_lossy(&value).to_string();
                    }
                }
                frames.last_mut().unwrap().p_glare.push(p_glare);
            }

            Ok(Event::Empty(ref e)) if inside_param && e.name() == QName(b"p_softfocus") => {
                // Extract attributes
                let mut p_softfocus = PSoftFocus {
                    intensity: String::new(),
                };
                for attr in e.attributes() {
                    let Attribute { key, value } = attr.unwrap();
                    if key == QName(b"intensity") {
                        p_softfocus.intensity = String::from_utf8_lossy(&value).to_string();
                    }
                }
                frames.last_mut().unwrap().p_softfocus.push(p_softfocus);
            }

            Ok(Event::Empty(ref e )) if inside_param && e.name() == QName(b"p_dof") => {
                // Extract attributes
                let mut p_dof = PDof {
                    focus_distance: String::new(),
                    near_distance: String::new(),
                    far_distance: String::new(),
                    blur_max_far: String::new(),
                    blur_edge: String::new(),
                };
                for attr in e.attributes() {
                    let Attribute { key, value } = attr.unwrap();
                    if key == QName(b"focusDistance") {
                        p_dof.focus_distance = String::from_utf8_lossy(&value).to_string();
                    } else if key == QName(b"nearDistance") {
                        p_dof.near_distance = String::from_utf8_lossy(&value).to_string();
                    } else if key == QName(b"farDistance") {
                        p_dof.far_distance = String::from_utf8_lossy(&value).to_string();
                    } else if key == QName(b"blurMaxFar") {
                        p_dof.blur_max_far = String::from_utf8_lossy(&value).to_string();
                    } else if key == QName(b"blurEdge") {
                        p_dof.blur_edge = String::from_utf8_lossy(&value).to_string();
                    }
                }
                frames.last_mut().unwrap().p_dof.push(p_dof);
            }



            // break the loop when reaching the end of the document
            Ok(Event::Eof) => break,

            _ => (),
        }

        buf.clear();
    }
    frames
}


pub fn create_glare_fcv_struct(frames: &Vec<Frame>, filename: &str) -> Box<dyn NuccStruct> {
    let glare_frames: Vec<&Frame> = frames
        .iter()
        .filter(|frame| frame.p_glare.len() > 0)
        .collect();

    let mut data = String::new();
    data.push_str(format!("FCURVE_TYPE_GLARE,\r\nFCURVE_INTERPOLATION_CONSTRAINT,\r\n{},\r\n", glare_frames.len().to_string()).as_str());
   
    for frame in glare_frames {
        if let Some(p_glare) = frame.p_glare.first() {
            let threshold = format!("{:0<8}",format!("{:.6}", p_glare.threshold.parse::<f32>().unwrap()));
            let subtraction_color = format!("{:0<8}",format!("{:.6}", p_glare.subtraction_color.parse::<f32>().unwrap()));
            let composition_intensity = format!("{:0<8}",format!("{:.6}", p_glare.composition_intensity.parse::<f32>().unwrap()));
            
            data.push_str(format!("{},{},{},{},\r\n", frame.no.parse::<u32>().unwrap() / 100, threshold, subtraction_color, composition_intensity).as_str());
        }
    }

    let struct_info = NuccStructInfo {
        chunk_name: filename.to_string() + "_glare",
        chunk_type: NuccChunkType::NuccChunkBinary.to_string(),
        filepath: format!("Z:/anm/{}/fcv/{}_glare.fcv", filename, filename).to_string(),
    };

    Box::new(NuccBinary {
        struct_info,
        version: 121,
        data: data.into_bytes(),
    })
    

    
}

pub fn create_brightrate_fcv_struct(frames: &Vec<Frame>, filename: &str) -> Box<dyn NuccStruct> {
    let brightrate_frames: Vec<&Frame> = frames
    .iter()
    .filter(|frame| frame.p_gp.len() > 0)
    .collect();



    let bgbout_len = brightrate_frames.iter().filter(|frame| frame.p_gp.iter().find(|p_gp| p_gp.name == "bgbout").is_some()).count();

    let mut data = String::new();
    data.push_str(format!("FCURVE_TYPE_BRIGHT_RATE,\r\nFCURVE_INTERPOLATION_CONSTRAINT,\r\n{},\r\n", bgbout_len.to_string()).as_str());

    for frame in brightrate_frames {
        if let Some(p_gp) = frame.p_gp.iter().find(|p_gp| p_gp.name == "bgbout"){
            let value = format!("{:0<8}",format!("{:.6}", p_gp.value.split(',').next().unwrap().parse::<f32>().unwrap()));
            data.push_str(format!("{},{},\r\n", frame.no.parse::<u32>().unwrap() / 100, value).as_str());
        }
    }


    let struct_info = NuccStructInfo {
        chunk_name: filename.to_string() + "_bright_rate",
        chunk_type: NuccChunkType::NuccChunkBinary.to_string(),
        filepath: format!("Z:/anm/{}/fcv/{}_bright_rate.fcv", filename, filename).to_string(),
    };

    Box::new(NuccBinary {
        struct_info,
        version: 121,
        data: data.into_bytes(),
    })
}


pub fn create_dof_fcv_struct(frames: &Vec<Frame>, filename: &str) -> Box<dyn NuccStruct> {
    let dof_frames: Vec<&Frame> = frames
        .iter()
        .filter(|frame| frame.p_dof.len() > 0)
        .collect();

    let mut data = String::new();
    data.push_str(format!("FCURVE_TYPE_DOF,\r\nFCURVE_INTERPOLATION_CONSTRAINT,\r\n{},\r\n", dof_frames.len().to_string()).as_str());

    for frame in dof_frames {
        if let Some(p_dof) = frame.p_dof.first() {
            let focus_distance = format!("{:0<8}",format!("{:.6}", p_dof.focus_distance.parse::<f32>().unwrap()));
            let near_distance = format!("{:0<8}",format!("{:.6}", p_dof.near_distance.parse::<f32>().unwrap()));
            let far_distance = format!("{:0<8}",format!("{:.6}", p_dof.far_distance.parse::<f32>().unwrap()));
            let blur_max_far = format!("{:0<8}",format!("{:.6}", p_dof.blur_max_far.parse::<f32>().unwrap()));
            let blur_edge = format!("{:0<8}",format!("{:.6}", p_dof.blur_edge.parse::<f32>().unwrap()));
            
            data.push_str(format!("{},{},{},{},{},{},\r\n", frame.no.parse::<u32>().unwrap() / 100, focus_distance, near_distance, far_distance, blur_max_far, blur_edge).as_str());
        }
    }

    let struct_info = NuccStructInfo {
        chunk_name: filename.to_string() + "_dof",
        chunk_type: NuccChunkType::NuccChunkBinary.to_string(),
        filepath: format!("Z:/anm/{}/fcv/{}_dof.fcv", filename, filename).to_string(),
    };

    Box::new(NuccBinary {
        struct_info,
        version: 121,
        data: data.into_bytes(),
    })
}


pub fn create_softfocus_fcv_struct(frames: &Vec<Frame>, filename: &str) -> Box<dyn NuccStruct> {
    let softfocus_frames: Vec<&Frame> = frames
        .iter()
        .filter(|frame| frame.p_softfocus.len() > 0)
        .collect();

    let mut data = String::new();
    data.push_str(format!("FCURVE_TYPE_SOFTFOCUS,\r\nFCURVE_INTERPOLATION_CONSTRAINT,\r\n{},\r\n", softfocus_frames.len().to_string()).as_str());
   
    for frame in softfocus_frames {
        if let Some(p_softfocus) = frame.p_softfocus.first() {
            let intensity = format!("{:0<8}",format!("{:.6}", p_softfocus.intensity.parse::<f32>().unwrap()));
            data.push_str(format!("{},{},\r\n", frame.no.parse::<u32>().unwrap() / 100, intensity).as_str());
        }
    }

    let struct_info = NuccStructInfo {
        chunk_name: filename.to_string() + "_softfocus",
        chunk_type: NuccChunkType::NuccChunkBinary.to_string(),
        filepath: format!("Z:/anm/{}/fcv/{}_softfocus.fcv", filename, filename).to_string(),
    };

    Box::new(NuccBinary {
        struct_info,
        version: 121,
        data: data.into_bytes(),
    })
}

pub fn create_zrange_fcv_struct(filename: &str) -> Box<dyn NuccStruct> {
 
    let mut data = String::new();
    data.push_str(format!("FCURVE_TYPE_ZRANGE,\r\nFCURVE_INTERPOLATION_CONSTRAINT,\r\n1,\r\n").as_str());
    data.push_str(format!("1,1.000000,9000000.000000\r\n").as_str());
  

    let struct_info = NuccStructInfo {
        chunk_name: filename.to_string() + "_zrange",
        chunk_type: NuccChunkType::NuccChunkBinary.to_string(),
        filepath: format!("Z:/anm/{}/fcv/{}_zrange.fcv", filename, filename).to_string(),
    };

    Box::new(NuccBinary {
        struct_info,
        version: 121,
        data: data.into_bytes(),
    })
}


pub fn create_bcadjustments_fcv_struct(filename: &str) -> Box<dyn NuccStruct> {
 
    let mut data = String::new();
    data.push_str(format!("FCURVE_TYPE_BCADJUSTMENTS,\r\nFCURVE_INTERPOLATION_CONSTRAINT,\r\n1,\r\n").as_str());
    data.push_str(format!("0,0.00000,1.250000\r\n").as_str());
  

    let struct_info = NuccStructInfo {
        chunk_name: filename.to_string() + "_bcadjustments",
        chunk_type: NuccChunkType::NuccChunkBinary.to_string(),
        filepath: format!("Z:/anm/{}/fcv/{}_bcadjustments.fcv", filename, filename).to_string(),
    };

    Box::new(NuccBinary {
        struct_info,
        version: 121,
        data: data.into_bytes(),
    })
}


pub fn create_fcv_xfbin(xfbin: &mut Xfbin, frames: &Vec<Frame>, filename: &str) {
    if frames.iter().find(|frame| frame.p_glare.len() > 0).is_some() {
        let glare_fcv = create_glare_fcv_struct(frames, filename);
        let mut glare_page = XfbinPage::default();
        glare_page.structs.push(glare_fcv);
        xfbin.pages.push(glare_page);
    }

    if frames.iter().find(|frame| frame.p_softfocus.len() > 0).is_some() {
        let softfocus_fcv = create_softfocus_fcv_struct(frames, filename);
        let mut softfocus_page = XfbinPage::default();
        softfocus_page.structs.push(softfocus_fcv);
        xfbin.pages.push(softfocus_page);
    }

    if frames.iter().find(|frame| frame.p_dof.len() > 0).is_some() {
        let dof_fcv = create_dof_fcv_struct(frames, filename);
        let mut dof_page = XfbinPage::default();
        dof_page.structs.push(dof_fcv);
        xfbin.pages.push(dof_page);
    }

    // For bright rate we only push ones that have "bgbout" as a parameter if they exist
    if frames.iter().find(|frame| frame.p_gp.iter().find(|p_gp| p_gp.name == "bgbout").is_some()).is_some() {
        let bright_rate_fcv = create_brightrate_fcv_struct(frames, filename);
        let mut bright_rate_page = XfbinPage::default();
        bright_rate_page.structs.push(bright_rate_fcv);
        xfbin.pages.push(bright_rate_page);
    }

    let zrange_fcv = create_zrange_fcv_struct(filename);
    let mut zrange_page = XfbinPage::default();
    zrange_page.structs.push(zrange_fcv);

    let bcadjustments_fcv = create_bcadjustments_fcv_struct(filename);
    let mut bcadjustments_page = XfbinPage::default();
    bcadjustments_page.structs.push(bcadjustments_fcv);

    xfbin.pages.push(zrange_page);
    xfbin.pages.push(bcadjustments_page);
    
}