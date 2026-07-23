    {
        vec3 flo = (c + d_cross_avg * 4.0) * 0.2;
        vec3 fhi = c - flo;
        vec3 fhl = (history + hist_e + hist_w + hist_n + hist_s) * 0.2;
        vec3 fhhi = history - fhl;
        vec3 ff_out = mix(flo, fhl, 0.7) + mix(fhi, fhhi, 0.2);
        c = mix(c, ff_out, hist_valid);
    }
