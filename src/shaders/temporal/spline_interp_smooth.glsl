    {
        vec3 p0 = history - (c - history);
        vec3 p3 = c + (c - history);
        vec3 m1 = 0.5 * (c - p0);
        vec3 m2 = 0.5 * (p3 - history);
        const float h00 =  0.5;
        const float h10 =  0.125;
        const float h01 =  0.5;
        const float h11 = -0.125;
        vec3 ch = h00 * history + h10 * m1 + h01 * c + h11 * m2;
        c = mix(c, ch, hist_valid);
    }
