function isVisible(element) {
    var coordinates = element.getBoundingClientRect();

    if (
    coordinates.right > window.innerWidth ||
    coordinates.bottom > window.innerHeight
    ) {
    return false;
    }

    if (coordinates.top < 0 || coordinates.left < 0) {
    return false;
    }

    return true;
}