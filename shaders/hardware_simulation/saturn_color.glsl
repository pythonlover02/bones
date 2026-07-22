    {
        float sat_lum = dot(c, LUMA_BT601);
        float sat_qlum = floor(sat_lum * 12.0 + 0.5) * 0.08333333;
        c = c * (sat_qlum / max(sat_lum, 0.0001));
        c = c * 0.85;
        float sat_g = dot(c, LUMA_BT601);
        c = mix(vec3(sat_g), c, 0.75);
        c = c * vec3(1.05, 0.97, 0.85);
        c = floor(c * 31.0 + 0.5 + bayer_signed(frag_coord) * 0.02) * 0.03225806;
    }
