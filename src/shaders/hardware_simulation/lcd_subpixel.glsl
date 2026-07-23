    {
        float lc = 3.0 * res_scale;
        vec2 lp = fract(frag_coord / lc);
        float lm = step(0.2, lp.x) * step(0.2, lp.y);
        c = c * mix(0.6, 1.0, lm) * 1.1;
    }
