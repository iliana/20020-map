require('lodash.combinations');

const Cesium = require('cesium');
const _ = require('lodash');
const fs = require('fs');
const parse = require('csv-parse');
const path = require('path');
const process = require('process');
const util = require('util');
const xml2js = require('xml2js');
const xpath = require('xml2js-xpath');

const HASH_MARK_DISTANCE = 40 / 3.2808399;
const TWENTY_YARDS = 60 / 3.2808399;
const TWO_FEET = 2 / 3.2808399;

function mean(x) {
  return x.reduce((acc, n) => acc + n, 0) / x.length;
}

// https://mathworld.wolfram.com/MercatorProjection.html
function mercatorProject(point) {
  return new Cesium.Cartesian3(point.longitude, Math.asinh(Math.tan(point.latitude)), 0);
}

function mercatorUnproject(point) {
  return new Cesium.Cartographic(point.x, Math.atan(Math.sinh(point.y)), 0);
}

// https://stackoverflow.com/a/565282
function intersection(l, m) {
  function cross(v, w) { return v.x * w.y - v.y * w.x; }
  function add(v, w) { return Cesium.Cartesian3.add(v, w, Cesium.Cartesian3.ZERO.clone()); }
  function sub(v, w) { return Cesium.Cartesian3.subtract(v, w, Cesium.Cartesian3.ZERO.clone()); }
  function scale(v, x) {
    return Cesium.Cartesian3.multiplyByScalar(v, x, Cesium.Cartesian3.ZERO.clone());
  }

  const p = mercatorProject(l.start);
  const q = mercatorProject(m.start);
  const r = sub(mercatorProject(l.end), p);
  const s = sub(mercatorProject(m.end), q);

  const rs = cross(r, s);
  if (Cesium.Math.equalsEpsilon(rs, 0, Cesium.Math.EPSILON6)) { return undefined; }
  const qp = sub(q, p);
  const t = cross(qp, s) / rs;
  const u = cross(qp, r) / rs;
  return (t >= 0 && t <= 1 && u >= 0 && u <= 1)
    ? mercatorUnproject(add(p, scale(r, t))) : undefined;
}

function shortestRhumb(start, heading, points) {
  return new Cesium.EllipsoidRhumbLine(...[heading - Cesium.Math.PI, heading]
    .map((h) => Cesium.EllipsoidRhumbLine.fromStartHeadingDistance(start, h,
      Math.min(...points
        .filter((point) => Cesium.Math.equalsEpsilon(
          Cesium.Math.negativePiToPi(h),
          Cesium.Math.negativePiToPi(new Cesium.EllipsoidRhumbLine(start, point).heading),
          Cesium.Math.EPSILON2,
        ))
        .map((point) => new Cesium.EllipsoidRhumbLine(start, point).surfaceDistance))).end));
}

async function loadSurvey(team, boundary) {
  const filepath = path.join(__dirname, 'survey', `${team.team}.kml`);
  if (!await util.promisify(fs.exists)(filepath)) {
    return undefined;
  }

  const xml = await util.promisify(fs.readFile)(filepath)
    .then(xml2js.parseStringPromise);
  const points = xpath.find(xml, '//Point/coordinates')
    .map((point) => Cesium.Cartographic.fromDegrees(...point.split(',').slice(0, 2).map(parseFloat)));

  // Find the lines between the points that are parallel (twenty yards distance) and perpendicular
  // (forty feet distance, between the hash marks) to determine the heading of the field
  const lines = _.combinations(points, 2).map(([a, b]) => new Cesium.EllipsoidRhumbLine(a, b));
  const heading = mean([
    ...lines
      .filter((line) => Math.abs(line.surfaceDistance - HASH_MARK_DISTANCE) < TWO_FEET)
      .map((line) => line.heading + Cesium.Math.toRadians(90)),
    ...lines
      .filter((line) => Math.abs(line.surfaceDistance - TWENTY_YARDS) < TWO_FEET)
      .map((line) => line.heading),
  ].map((h) => Cesium.Math.mod(Cesium.Math.zeroToTwoPi(h), Cesium.Math.PI)));

  // Find the center of the field by finding the mean of the survey points
  const field = new Cesium.Cartographic(
    mean(points.map((p) => p.longitude)),
    mean(points.map((p) => p.latitude)),
  );

  // Create a rhumb line containing the field bounded by the boundary shape
  const tinyLine = Cesium.EllipsoidRhumbLine.fromStartHeadingDistance(field, heading, 100);
  const bigLine = shortestRhumb(field, heading, [
    ...boundary.longitude.map((lon) => tinyLine.findIntersectionWithLongitude(lon)),
    ...boundary.latitude.map((lat) => tinyLine.findIntersectionWithLatitude(lat)),
  ]);
  const line = shortestRhumb(field, heading, boundary.lines
    .map((l) => intersection(bigLine, l))
    .filter((point) => point !== undefined));

  return {
    ...team, field, heading, line,
  };
}

async function loadBoundary() {
  const points = await util.promisify(fs.readFile)(path.join(__dirname, 'data', 'boundary.csv'))
    .then((data) => util.promisify(parse)(data, { comment: '#' }))
    .then((data) => data.map((row) => Cesium.Cartographic.fromDegrees(...row.map(parseFloat))));
  return {
    longitude: [
      Math.min(...points.map((point) => point.longitude)) - Cesium.Math.EPSILON6,
      Math.max(...points.map((point) => point.longitude)) + Cesium.Math.EPSILON6,
    ],
    latitude: [
      Math.min(...points.map((point) => point.latitude)) - Cesium.Math.EPSILON6,
      Math.max(...points.map((point) => point.latitude)) + Cesium.Math.EPSILON6,
    ],
    lines: points.map((point, index) => new Cesium.EllipsoidRhumbLine(
      point,
      points[(index + 1) % points.length],
    )),
  };
}

async function main() {
  const boundary = await loadBoundary();
  await util.promisify(fs.writeFile)(
    path.join(__dirname, 'data', 'teams.json'),
    JSON.stringify(await util.promisify(fs.readFile)(path.join(__dirname, 'data', 'teams.csv'))
      .then((data) => util.promisify(parse)(data, { columns: true }))
      .then((ts) => Promise.all(ts.map((team) => loadSurvey(team, boundary))))
      .then((ts) => ts.filter((team) => team !== undefined))
      .then((ts) => ts.map((team) => ({
        ...team,
        color: Cesium.Color.pack(Cesium.Color.fromCssColorString(team.color), [], 0),
        line: [team.line.start.longitude, team.line.start.latitude,
          team.line.end.longitude, team.line.end.latitude],
      })))),
  );
}

main().catch((e) => {
  console.error(e);
  process.exitCode = 1;
});
