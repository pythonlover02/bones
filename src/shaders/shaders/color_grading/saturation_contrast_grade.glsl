    {
        float gl = dot(c, LUMA_BT601);
        c = mix(vec3(gl), c, 1.1);
        c = (c - HALF3) * 1.05 + HALF3;
    }
