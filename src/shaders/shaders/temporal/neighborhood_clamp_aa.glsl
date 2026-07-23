    {
        vec3 tmn = min(dmin_3x3, c);
        vec3 tmx = max(dmax_3x3, c);
        c = mix(c, mix(c, clamp(history, tmn, tmx), 0.9), hist_valid);
    }
