    {
        float psp_lum = dot(c, LUMA_BT601);
        float psp_qlum = floor(psp_lum * 16.0 + 0.5) * 0.0625;
        c = c * (psp_qlum / max(psp_lum, 0.0001));
        c = mix(vec3(0.06), vec3(0.92), c);
        float psp_dl = dot(c, LUMA_BT601);
        float psp_dark_q = mix(8.0, 31.0, smoothstep(0.0, 0.3, psp_dl));
        c = floor(c * psp_dark_q + 0.5) / psp_dark_q;
        c = floor(c * 31.0 + 0.5 + bayer_signed(frag_coord) * 0.018) * 0.03225806;
        float psp_g = dot(c, LUMA_BT601);
        c = mix(vec3(psp_g), c, 0.88);
    }
