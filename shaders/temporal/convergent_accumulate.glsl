    {
        vec3 ad = c - history;
        float am = dot(ad, ad);
        float as2 = 1.0 - clamp(am * 15.0, 0.0, 1.0);
        float aw = mix(0.1, 0.85, as2 * as2);
        c = mix(c, mix(c, history, aw), hist_valid);
    }
