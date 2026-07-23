    {
        float ll = dot(c, LUMA_BT709);
        float lw = mix(0.7, 0.15, smoothstep(0.0, 0.3, ll));
        c = mix(c, mix(c, history, lw), hist_valid);
    }
