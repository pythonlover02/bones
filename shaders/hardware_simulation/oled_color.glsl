    {
        float ol = dot(c, LUMA_BT601);
        float oc = smoothstep(0.0, 0.05, ol);
        c = (c + (c - vec3(ol)) * 0.1) * oc;
    }
