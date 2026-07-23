    {
        vec3 bca = (c + d_cross_avg * 4.0) * 0.2;
        vec3 bha = (history + hist_e + hist_w + hist_n + hist_s) * 0.2;
        vec3 bd = bca - bha;
        float bsm = 1.0 / (1.0 + dot(bd, bd) * 20.0);
        c = mix(c, mix(c, history, 0.5 * bsm), hist_valid);
    }
