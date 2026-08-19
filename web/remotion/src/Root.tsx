import React from "react";
import { Composition } from "remotion";
import { Banner } from "./Banner";
import { BannerV2 } from "./BannerV2";
import { MangaBrutal } from "./MangaBrutal";
import { FavIcon } from "./FavIcon";

export const RemotionRoot: React.FC = () => {
  return (
    <>
      <Composition
        id="Banner"
        component={Banner}
        durationInFrames={120}
        fps={30}
        width={960}
        height={320}
      />
      <Composition
        id="BannerV2"
        component={BannerV2}
        durationInFrames={120}
        fps={30}
        width={960}
        height={320}
      />
      <Composition
        id="MangaBrutal"
        component={MangaBrutal}
        durationInFrames={120}
        fps={30}
        width={960}
        height={320}
      />
      <Composition
        id="FavIcon"
        component={FavIcon}
        durationInFrames={1}
        fps={30}
        width={200}
        height={200}
      />
    </>
  );
};