    {
        float xb_lum = dot(c, LUMA_BT601);
        c = c + vec3(smoothstep(0.6, 0.9, xb_lum)) * 0.1;
        c = c + max(c - HALF3, ZERO3) * 0.12;
        c = c * vec3(1.02, 1.04, 0.96);
    }
