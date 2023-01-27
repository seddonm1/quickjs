// simple approximate distance function returning distance between two points in meters
function distance(lat0, lon0, lat1, lon1) {
    if ((lat0 == lat1) && (lon0 == lon1)) {
        return 0;
    } else {
        const radlat0 = Math.PI * lat0 / 180;
        const radlat1 = Math.PI * lat1 / 180;
        const theta = lon0 - lon1;
        const radtheta = Math.PI * theta / 180;
        let dist = Math.sin(radlat0) * Math.sin(radlat1) + Math.cos(radlat0) * Math.cos(radlat1) * Math.cos(radtheta);
        if (dist > 1) {
            dist = 1;
        }
        dist = Math.acos(dist);
        dist = dist * 180 / Math.PI;
        return dist * 60 * 1853.159;
    }
}

// calculate the total length of a set of input features in canadian football fields
function calculate(data) {
    // canadian football fields are 140 meters
    const candadian_football_field = 140;

    return data.features.reduce(
        (accumulator, currentValue, currentIndex, array) => {
            if (currentIndex == 0) {
                return 0
            } else {
                const previousValue = array[currentIndex - 1];
                const dist = distance(currentValue.geometry.coordinates[1], currentValue.geometry.coordinates[0], previousValue.geometry.coordinates[1], previousValue.geometry.coordinates[0]);
                return accumulator + dist / candadian_football_field
            }
        },
        0
    )
}

calculate(data)