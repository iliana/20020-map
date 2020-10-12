#!/usr/bin/python3

import xml.etree.ElementTree as ET
import sys
import re
import math
import itertools


# distance between two points in feet
def distance(x, y):
    x = (x[0] * math.pi / 180, x[1] * math.pi / 180)
    y = (y[0] * math.pi / 180, y[1] * math.pi / 180)
    a = math.cos(x[0]) * math.cos(y[0]) * math.pow(math.sin(abs(x[1] - y[1]) / 2), 2)
    a += math.pow(math.sin(abs(x[0] - y[0]) / 2), 2)
    c = 2 * math.atan2(math.sqrt(a), math.sqrt(1 - a))
    return 6.371e6 / 0.3048 * c


def bearing(a, b):
    a = (a[0] * math.pi / 180, a[1] * math.pi / 180)
    b = (b[0] * math.pi / 180, b[1] * math.pi / 180)
    d_lon = abs(a[1] - b[1])

    y = math.sin(d_lon) * math.cos(b[0])
    x = math.cos(a[0]) * math.sin(b[0])
    x -= math.sin(a[0]) * math.cos(b[0]) * math.cos(d_lon)
    return 180 / math.pi * math.atan2(y, x)


if __name__ == "__main__":
    tree = ET.parse(sys.argv[1])
    m = re.match("\{.*\}", tree.getroot().tag)
    ns = m.group(0) if m else ""

    points = []
    for point in tree.findall(f".//{ns}Point/{ns}coordinates"):
        point = point.text.split(",")
        points.append((float(point[1]), float(point[0])))
    print("lat =", round(sum(x[0] for x in points) / len(points), 6))
    print("lon =", round(sum(x[1] for x in points) / len(points), 6))

    angles = []
    for x, y in itertools.combinations(points, 2):
        d = distance(x, y)
        if abs(d - 60) < 2:
            angles.append(bearing(max(x, y), min(x, y)))
        elif abs(d - 40) < 2:
            angles.append((bearing(min(x, y), max(x, y)) + 90) % 180)

    if any(x < 90 for x in angles) and any(x > 90 for x in angles):
        angles = [x if x < 90 else x - 180 for x in angles]

    print("bearing =", round((sum(angles) / len(angles)) % 360, 6))
