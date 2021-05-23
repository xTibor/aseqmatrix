function distance(x1, y1, x2, y2)
    return math.sqrt(math.pow(x1 - x2, 2) + math.pow(y1 - y2, 2))
end

function linearInterpolation(a, b, t)
    return a * (1.0 - t) + b * t
end

-- http://paulbourke.net/miscellaneous/interpolation/
function catmullInterpolation(v0, v1, v2, v3, t)
    local a0 = -0.5 * v0 + 1.5 * v1 - 1.5 * v2 + 0.5 * v3
    local a1 =  1.0 * v0 - 2.5 * v1 + 2.0 * v2 - 0.5 * v3
    local a2 = -0.5 * v0 + 0.0 * v1 + 0.5 * v2 + 0.0 * v3
    local a3 =  0.0 * v0 + 1.0 * v1 + 0.0 * v2 + 0.0 * v3
    return (a0 * t * t * t) + (a1 * t * t) + (a2 * t) + (a3)
end

function love.load()
    x1, y1 = 100, 100
    x2, y2 = 300, 150
    xm, ym = 0, 0
    length = 300
end

function love.update(dt)
    if love.mouse.isDown(1) then
        x1, y1 = love.mouse.getPosition()
    elseif love.mouse.isDown(2) then
        x2, y2 = love.mouse.getPosition()
    end

    xm, ym = (x1 + x2) / 2, (y1 + y2) / 2
    d = distance(x1, y1, x2, y2)
    if d < length then
        ym = ym + math.sqrt((length * length - d * d) / 2)
    end
end

function love.wheelmoved(x, y)
    length = length + y
end

function love.draw()
    love.graphics.setColor(1.0, 0.0, 0.0)
    love.graphics.circle("fill", x1, y1, 10)
    love.graphics.setColor(0.0, 0.0, 1.0)
    love.graphics.circle("fill", x2, y2, 10)

    love.graphics.setColor(0.5, 0.5, 0.5)
    if distance(x1, y1, x2, y2) < length then
        local curve = {}
        local segmentCount = 10

        for i = 0, segmentCount - 1 do -- minus one to avoid duplicate points at (xm, ym)
            t = i / segmentCount
            x, y = linearInterpolation(x1, xm, t), catmullInterpolation(y1, y1, ym, y2, t)
            table.insert(curve, x)
            table.insert(curve, y)
        end

        for i = 0, segmentCount do
            t = i / segmentCount
            x, y = linearInterpolation(xm, x2, t), catmullInterpolation(y1, ym, y2, y2, t)
            table.insert(curve, x)
            table.insert(curve, y)
        end
        love.graphics.line(curve)
    else
        love.graphics.line(x1, y1, xm, ym)
        love.graphics.line(xm, ym, x2, y2)
    end

end
