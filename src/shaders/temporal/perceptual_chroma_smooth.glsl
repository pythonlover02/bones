    {
        vec3 pu = perc_ycc(c);
        vec3 ph = perc_ycc(history);
        c = mix(c, perc_rgb(mix(pu, ph, vec3(0.4, 0.7, 0.7))), hist_valid);
    }
