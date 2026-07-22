    {
        vec3 ed = c - history;
        float ev = dot(ed, ed);
        float ea = 0.6 / max(1.0 + ev * 50.0, 0.0001);
        c = mix(c, mix(c, history, clamp(ea, 0.0, 0.9)), hist_valid);
    }
