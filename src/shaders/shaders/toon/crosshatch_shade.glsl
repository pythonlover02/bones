    {
        const float HATCH_PITCH = 6.0;
        const float HATCH_LINE_LO = 0.6;
        const float HATCH_LINE_HI = 0.85;
        const float HATCH_MID_LO = 0.45;
        const float HATCH_MID_HI = 0.6;
        const float HATCH_DEEP_LO = 0.2;
        const float HATCH_DEEP_HI = 0.35;
        const float HATCH_DARKEN = 0.3;
        float hatch_l = dot(c, LUMA_BT709);
        float hatch_p = HATCH_PITCH * res_scale;
        float hatch_t1 = abs(fract((frag_coord.x + frag_coord.y) / hatch_p) - 0.5) * 2.0;
        float hatch_t2 = abs(fract((frag_coord.x - frag_coord.y) / hatch_p) - 0.5) * 2.0;
        float hatch_l1 = smoothstep(HATCH_LINE_LO, HATCH_LINE_HI, hatch_t1) * (1.0 - smoothstep(HATCH_MID_LO, HATCH_MID_HI, hatch_l));
        float hatch_l2 = smoothstep(HATCH_LINE_LO, HATCH_LINE_HI, hatch_t2) * (1.0 - smoothstep(HATCH_DEEP_LO, HATCH_DEEP_HI, hatch_l));
        c = mix(c, c * HATCH_DARKEN, clamp(hatch_l1 + hatch_l2, 0.0, 1.0));
    }
