    {
        const float TONE_CELL = 4.5;
        const float TONE_DIAG = 0.70710678;
        const float TONE_RADIUS = 0.62;
        const float TONE_SOFT = 0.1;
        const float TONE_ZONE_LO = 0.45;
        const float TONE_ZONE_HI = 0.7;
        const float TONE_DARKEN = 0.2;
        float tone_ts = TONE_CELL * res_scale;
        vec2 tone_p = vec2(frag_coord.x + frag_coord.y, frag_coord.x - frag_coord.y) * (TONE_DIAG / tone_ts);
        vec2 tone_cell = fract(tone_p) - HALF2;
        float tone_l = dot(c, LUMA_BT709);
        float tone_r = (1.0 - tone_l) * TONE_RADIUS;
        float tone_dm = 1.0 - smoothstep(tone_r - TONE_SOFT, tone_r, length(tone_cell));
        float tone_zone = 1.0 - smoothstep(TONE_ZONE_LO, TONE_ZONE_HI, tone_l);
        c = mix(c, c * TONE_DARKEN, tone_dm * tone_zone);
    }
