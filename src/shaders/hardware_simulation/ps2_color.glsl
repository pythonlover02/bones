    {
        const vec3 PS2_THR = vec3(0.6);
        float ps2_scanline = floor(frag_coord.y);
        float ps2_sc = 0.97 + 0.03 * (abs(fract(ps2_scanline * 0.5) - 0.5) * 4.0 - 1.0);
        c = c * ps2_sc;
        c = c + max(c - PS2_THR, ZERO3) * 0.15;
    }
