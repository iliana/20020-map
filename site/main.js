/* global Cesium */

Cesium.Camera.DEFAULT_VIEW_RECTANGLE = Cesium.Rectangle.fromDegrees(
  -124.725839, 26.169463,
  -66.949895, 50.884358,
);
Cesium.Camera.DEFAULT_VIEW_FACTOR = 0.01;

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
  timeline: false,
});

viewer.dataSources.add(Cesium.KmlDataSource.load('20020.kml', {
  camera: viewer.camera,
  canvas: viewer.canvas,
  credit: new Cesium.Credit(
    '<a href="https://www.sbnation.com/secret-base/21410129/20020"><i>20020</i></a> '
    + 'by Secret Base. © 2020 Vox Media, Inc. All Rights Reserved. '
    + 'Author, Illustrator, Video Director: Jon Bois; '
    + 'Editor: Graham MacAree; Engineer: Frank Bi. '
    + 'Data: Google, Landsat/Copernicus, LDEO-Columbia, NSF, NOAA, SIO, U.S. Navy, NGA, GEBCO'
  ),
}));
