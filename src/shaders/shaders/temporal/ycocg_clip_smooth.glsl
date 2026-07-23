    {
        vec3 fh = ycocg_encode(history);
        vec3 fu = ycocg_encode(c);
        vec3 fe = ycocg_encode(dtap_1_0);
        vec3 fw = ycocg_encode(dtap_m1_0);
        vec3 fn = ycocg_encode(dtap_0_1);
        vec3 fs = ycocg_encode(dtap_0_m1);
        vec3 fmn = min(fu, min(min(fe, fw), min(fn, fs)));
        vec3 fmx = max(fu, max(max(fe, fw), max(fn, fs)));
        vec3 fcl = clamp(fh, fmn, fmx);
        vec3 fdc = abs(fh - fcl);
        float ff = 1.0 - clamp((fdc.x + fdc.y + fdc.z) * 2.0, 0.0, 1.0);
        c = mix(c, ycocg_decode(mix(fu, fcl, 0.7 * ff)), hist_valid);
    }
