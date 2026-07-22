    {
        const float CEL_BANDS = 4.0;
        const float CEL_EDGE_LO = 0.35;
        const float CEL_EDGE_HI = 0.65;
        const float CEL_FLOOR = 0.06;
        float cel_l = dot(c, LUMA_BT709);
        float cel_b = cel_l * CEL_BANDS;
        float cel_q = max((floor(cel_b) + smoothstep(CEL_EDGE_LO, CEL_EDGE_HI, fract(cel_b))) / CEL_BANDS, CEL_FLOOR);
        c = c * (cel_q / max(cel_l, 0.0001));
    }
