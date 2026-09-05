import React from "react";
import { Composition } from "remotion";
import { Banner } from "./Banner";
import { BannerV2 } from "./BannerV2";
import { BannerV3 } from "./BannerV3";
import { DemoTerminal } from "./DemoTerminal";
import { MangaBrutal } from "./MangaBrutal";
import { FavIcon } from "./FavIcon";
import { OrgBadge } from "./OrgBadge";
import { OrgMark } from "./OrgMark";

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
        id="BannerV3"
        component={BannerV3}
        durationInFrames={120}
        fps={30}
        width={960}
        height={320}
      />
      <Composition
        id="DemoTerminal"
        component={DemoTerminal}
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
      <Composition
        id="OrgBadge"
        component={OrgBadge}
        durationInFrames={120}
        fps={30}
        width={960}
        height={320}
      />
      <Composition
        id="OrgBadgeLight"
        component={OrgBadge}
        durationInFrames={120}
        fps={30}
        width={960}
        height={320}
        defaultProps={{ theme: "light" }}
      />
      <Composition
        id="OrgMark"
        component={OrgMark}
        durationInFrames={120}
        fps={30}
        width={960}
        height={240}
      />
      <Composition
        id="OrgMarkLight"
        component={OrgMark}
        durationInFrames={120}
        fps={30}
        width={960}
        height={240}
        defaultProps={{ theme: "light" }}
      />
    </>
  );
};