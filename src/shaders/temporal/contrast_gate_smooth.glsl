    {
        float cl = dot(c, LUMA_BT709);
        float cn = dot(d_cross_avg, LUMA_BT709);
        float cw = mix(0.7, 0.1, clamp(abs(cl - cn) * 8.0, 0.0, 1.0));
        c = mix(c, mix(c, history, cw), hist_valid);
    }
