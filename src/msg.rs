use msgcont::{self, Decode};

#[derive(Debug)]
pub enum TrendMsg{
    Daq{content:msgcont::Daq<[u32;2]>},
    Trig{content:msgcont::Trig<[u32;4]>},
    SlcReq,
    Gps{content:msgcont::Gps<[u32;1]>, payload:Vec<u8>},
    Adc{content:msgcont::Adc<[u32;1]>},
    IntReg{content:msgcont::IntReg<[u32;11]>},
    Data{content:msgcont::Data<[u32;5]>, payload:Vec<u16>},
    Slc{content:msgcont::Slc<[u32;16]>},
    RdIntReg{content:msgcont::RdIntReg<[u32;13]>},
    Ack{content:msgcont::Ack<[u32;2]>},
}



impl TrendMsg{
    pub fn header()->u32{
        0xaaaaaaaa
    }

    pub fn tailer()->u32{
        0xaaaaaaaa
    }

    pub fn type_code(&self)->u32{
        match self{
            &TrendMsg::Daq{..}=>0x5000,
            &TrendMsg::Trig{..}=>0x5100,
            &TrendMsg::SlcReq{..}=>0x5200,
            &TrendMsg::Gps{..}=>0x5300,
            &TrendMsg::Adc{..}=>0x5400,
            &TrendMsg::IntReg{..}=>0x5E00,
            &TrendMsg::Data{..}=>0x5A00,
            &TrendMsg::Slc{..}=>0x5B00,
            &TrendMsg::RdIntReg{..}=>0x5C00,
            &TrendMsg::Ack{..}=>0x5D00,
        }
    }

    pub fn type_name(&self)->&str{
        match self{
            &TrendMsg::Daq{..}=>"DAQ",
            &TrendMsg::Trig{..}=>"TRIG",
            &TrendMsg::SlcReq{..}=>"SLCREQ",
            &TrendMsg::Gps{..}=>"GPS",
            &TrendMsg::Adc{..}=>"ADC",
            &TrendMsg::IntReg{..}=>"INTREG",
            &TrendMsg::Data{..}=>"DATA",
            &TrendMsg::Slc{..}=>"SLC",
            &TrendMsg::RdIntReg{..}=>"RDINTREG",
            &TrendMsg::Ack{..}=>"ACK",
        }
    }

    pub fn get_content_pulp(&self)->Option<&[u32]>{
        match self{
            &TrendMsg::Daq{ref content,..}=>Some(&content.0),
            &TrendMsg::Trig{ref content,..}=>{
                match content.cntrl_dac()&0x80_00{
                    1 => Some(&content.0[0..1]),
                    _ => Some(&content.0)
                }
            },
            &TrendMsg::SlcReq{..}=>None,
            &TrendMsg::Gps{ref content,..}=>Some(&content.0),
            &TrendMsg::Adc{ref content,..}=>Some(&content.0),
            &TrendMsg::IntReg{ref content,..}=>{
                match content.y(){
                    0 => Some(&content.0[0..1]),
                    _ => Some(&content.0)
                }
            },
            &TrendMsg::Data{ref content,..}=>Some(&content.0),
            &TrendMsg::Slc{ref content,..}=>Some(&content.0),
            &TrendMsg::RdIntReg{ref content,..}=>Some(&content.0),
            &TrendMsg::Ack{ref content,..}=>Some(&content.0),
        }
    }

    pub fn get_payload_word_vec(&self)->Option<Vec<u32>>{
        match self{
            &TrendMsg::Gps{ref payload,..} =>{
                let mut result=vec![0;(payload.len()+1)/2];
                for i in 0..payload.len(){
                    let out_id=i/2;
                    if i%2==0{
                        result[out_id]+=payload[i] as u32;
                    }
                    else{
                        result[out_id]+=(payload[i] as u32)<<8;
                    }
                }
                Some(result)
            }
            &TrendMsg::Data{ref payload,..} =>{
                let mut result=vec![0;(payload.len()+1)/2];
                for i in 0..payload.len(){
                    let out_id=i/2;
                    if i%2==0{
                        result[out_id]+=payload[i] as u32;
                    }
                    else{
                        result[out_id]+=(payload[i] as u32)<<12;
                    }
                }
                Some(result)
            }
            _=>None
        }
    }

    pub fn to_word_vec(&self)->Vec<u32>{
        let mut result=vec![Self::header(), self.type_code()];
        if let Some(content_pulp)=self.get_content_pulp(){
            result.extend_from_slice(content_pulp);
        }
        if let Some(mut payload)=self.get_payload_word_vec(){
            result.append(&mut payload);
        }
        result.push(Self::tailer());
        result
    }

    pub fn from_word_vec(data:Vec<u32>)->Option<TrendMsg>{
        assert!(data[0]==Self::header());
        assert!(data[data.len()-1]==Self::tailer());
        match data[1]{
            0x5000 => msgcont::Daq::<[u32;2]>::decode(&data[2..]).map(|x| TrendMsg::Daq{content:x}),
            0x5100 => msgcont::Trig::<[u32;4]>::decode(&data[2..]).map(|x| TrendMsg::Trig{content:x}),
            0x5200 => Some(TrendMsg::SlcReq),
            0x5300 => {
                if data.len()<4{
                    None
                }
                else{
                    let mut payload=Vec::with_capacity((data.len()-4)*2);
                    for d in &data[3..data.len()-1]{
                        payload.push((*d&0xff) as u8);
                        payload.push(((*d>>8)&0xff) as u8);
                    }
                    msgcont::Gps::<[u32;1]>::decode(&data[2..]).map(|x| TrendMsg::Gps{content:x, payload:payload})
                }
            }
            0x5400 => msgcont::Adc::<[u32;1]>::decode(&data[2..]).map(|x|TrendMsg::Adc{content:x}),
            0x5E00 => msgcont::IntReg::<[u32;11]>::decode(&data[2..]).map(|x|TrendMsg::IntReg{content:x}),
            0x5A00 => {
                if data.len()<8{
                    None
                }
                else{
                    let mut payload=Vec::with_capacity((data.len()-8)*2);
                    for d in &data[7..data.len()-1]{
                        payload.push((*d&0xfff) as u16);
                        payload.push(((*d>>12)&0xfff) as u16);
                    }
                    msgcont::Data::<[u32;5]>::decode(&data[2..]).map(|x|TrendMsg::Data{content:x, payload:payload})
                }
            }
            0x5B00 => msgcont::Slc::<[u32;16]>::decode(&data[2..]).map(|x|TrendMsg::Slc{content:x}),
            0x5C00 => msgcont::RdIntReg::<[u32;13]>::decode(&data[2..]).map(|x|TrendMsg::RdIntReg{content:x}),
            0x5D00 => msgcont::Ack::<[u32;2]>::decode(&data[2..]).map(|x|TrendMsg::Ack{content:x}),
            _=>None
        }
    }

    pub fn to_byte_vec(&self)->Vec<u8>{
        let word_slice=self.to_word_vec().into_boxed_slice();
        let cap=word_slice.len()*4;
        unsafe{Vec::from_raw_parts(Box::into_raw(word_slice) as *mut u8, cap, cap)}
    }

    pub fn from_byte_vec(data:Vec<u8>)->Option<TrendMsg>{
        let word_cap=data.len()/4;
        assert!(word_cap*4==data.len());
        Self::from_word_vec(
            unsafe{
                Vec::from_raw_parts(Box::into_raw(data.into_boxed_slice()) as *mut u32, word_cap, word_cap)
                })
    }
}
