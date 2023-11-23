#[repr(C)]
pub struct AplicDomain {
    domain_cfg: DomainCfg,
    src_cfg: [u32; 1024],
    mmsi_addr_cfg: u32,
    mmsi_addr_cfgh: u32,
    smsi_addr_cfg: u32,
    smsi_addr_cfgh: u32,
    set_ip: [u32; 32],
    set_ip_num: u32,
    in_clr_ip: [u32; 32],
    clr_ip_num: u32,
    set_ie: [u32; 32],
    set_ie_num: u32,
    clr_ie: [u32; 32],
    clr_ie_num: u32,
    set_ip_num_le: u32,
    set_ip_num_be: libsa::endian::BigEndianU32,
    gen_msi: u32,
    target: [u32; 1024],
}

bitfield::bitfield! {
    #[repr(transparent)]
    pub struct DomainCfg(u32);
    impl Debug;
    
    pub big_endian, set_big_endian: 0;
    pub msi, set_msi: 2;
    pub int_enable, set_int_enable: 8;
}

bitfield::bitfield! {
    #[repr(transparent)]
    pub struct SourceConfig(u32);
    impl Debug;

    /// 0: Inactive
    /// 1: Detatched
    /// 4: Rising edge
    /// 5: Falling edge
    /// 6: Level high
    /// 7: Level low
    pub src_mode, set_src_mode: 2, 0;
    pub child_idx, set_child_idx: 9, 0;
    pub d, _: 10;
}