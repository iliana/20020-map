/* global Cesium */

Cesium.Camera.DEFAULT_VIEW_RECTANGLE = Cesium.Rectangle.fromDegrees(
  -124.725839, 24.669463,
  -66.949895, 49.384358,
);
Cesium.Camera.DEFAULT_VIEW_FACTOR = 0.1;

// Remove Cesium ion logo as we are not using any ion services (we are crediting use of the library
// in text below)
// TODO: enforce we are not using ion services with Content-Security-Policy
Cesium.CreditDisplay.cesiumCredit = new Cesium.Credit();

const viewer = new Cesium.Viewer('map', {
  animation: false,
  baseLayerPicker: false,
  geocoder: false,
  imageryProvider: new Cesium.ArcGisMapServerImageryProvider({
    url: 'https://services.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer',
  }),
  infoBox: false,
  requestRenderMode: true,
  sceneModePicker: false,
  terrainProvider: new Cesium.ArcGISTiledElevationTerrainProvider({
    url: 'https://elevation3d.arcgis.com/arcgis/rest/services/WorldElevation3D/Terrain3D/ImageServer',
  }),
  timeline: false,
});

viewer.scene.frameState.creditDisplay.addDefaultCredit(new Cesium.Credit(
  '<a href="https://www.sbnation.com/secret-base">Secret Base</a>, '
  + '<a href="https://cesium.com/">Cesium</a>, '
  + 'Google, Landsat/Copernicus, LDEO-Columbia, NSF, NOAA, SIO, U.S. Navy, NGA, GEBCO',
));

// Disables camera-terrain collision detection for improved performance
// https://community.cesium.com/t/why-arcgis-tile-terrain-make-my-globe-is-super-slow/8899/7
viewer.scene.screenSpaceCameraController.enableCollisionDetection = false;

fetch('/data/teams.json')
  .then((response) => {
    if (!response.ok) {
    }
  })
