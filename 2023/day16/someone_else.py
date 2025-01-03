from time import time


def timer_func(func):
    # This function shows the execution time of
    # the function object passed
    def wrap_func(*args, **kwargs):
        t1 = time()
        result = func(*args, **kwargs)
        t2 = time()
        print(f'Function {func.__name__!r} executed in {(t2 - t1):.4f}s')
        return result

    return wrap_func


class Grid2d:

    def __init__(self, grid):
        self.grid = grid
        self.height = len(grid)
        self.width = len(grid[0])

    def __getitem__(self, item):
        # returns None if outside the grid
        if len(item) > 2:
            raise ValueError('Grid2D expected a list like of length 2. Length of accessor longer than expected')
        x, y = item
        if 0 <= x < self.height:
            if 0 <= y < self.width:
                return self.grid[x][y]
        return None


class LaserBeam:

    def __init__(self, pos, heading):
        self.pos = pos
        self.heading = heading

    def next_loc(self):
        x, y = self.pos
        h = self.heading
        if h == 'u':
            return x - 1, y
        elif h == 'd':
            return x + 1, y
        elif h == 'l':
            return x, y - 1
        elif h == 'r':
            return x, y + 1


class BeamMap:

    def __init__(self, grid, start=((0, -1), 'r')):
        self.grid = Grid2d(grid)
        self.beams = [LaserBeam(*start)]
        self.energized = set()  # set of coordinates (tuple) that are energized (x, y)
        self.beam_tracking = set()  # set of coordinates and heading of locations beams have gone through ((x, y), d)
        self.BOUNCE_DICT = {'l': {'/': ('d', None),
                                  '\\': ('u', None),
                                  '-': ('l', None),
                                  '|': ('u', 'd'),
                                  '.': ('l', None)},
                            'r': {'/': ('u', None),
                                  '\\': ('d', None),
                                  '-': ('r', None),
                                  '|': ('u', 'd'),
                                  '.': ('r', None)},
                            'u': {'/': ('r', None),
                                  '\\': ('l', None),
                                  '-': ('l', 'r'),
                                  '|': ('u', None),
                                  '.': ('u', None)},
                            'd': {'/': ('l', None),
                                  '\\': ('r', None),
                                  '-': ('l', 'r'),
                                  '|': ('d', None),
                                  '.': ('d', None)}}

    def project_beams(self):
        # list for new beams that form off splitters
        new_beams = []
        # list for beams the leave the grid or go into a loop
        beams_left_grid = []
        while self.beams:
            # clear the new beams and deleted beams list
            new_beams.clear()
            beams_left_grid.clear()
            # loop over the current beams
            for i, beam in enumerate(self.beams):
                # get the next location for the beam
                nl = beam.next_loc()
                # get the value of the grid
                ng = self.grid[nl]
                # if the next point is in the grid
                if ng:
                    # if the next point is a mirror or splitter, not empty space
                    if ng != '.':
                        # update the beam position
                        beam.pos = nl
                        # add to the energized set
                        self.energized.add(nl)
                        # find the new heading(s)
                        beam.heading, split_b = self.BOUNCE_DICT[beam.heading][ng]
                        # check to see if the current beam heading has been seen before
                        if self.beam_tracking.intersection([(nl, beam.heading)]):
                            # beam has been seen before and will start looping, add the delete list
                            beams_left_grid.append(i)
                        else:
                            # beam hasn't been seen before, add the record to the tracking set
                            self.beam_tracking.add((nl, beam.heading))
                        # if a split exists, and it isn't in the beam tracking set
                        if split_b and not self.beam_tracking.intersection([(nl, split_b)]):
                            # add the new beam to the new beam list
                            new_beams.append(LaserBeam(nl, split_b))
                            # add the new beam to the beam tracking set
                            self.beam_tracking.add((nl, split_b))
                    # beam position is empty space
                    else:
                        # update beam position
                        beam.pos = nl
                        # add to energized list
                        self.energized.add(nl)
                        # add to beam tracking list
                        self.beam_tracking.add((nl, beam.heading))
                # point is outside the grid
                else:
                    # add to delete list
                    beams_left_grid.append(i)
            # deleting beams that moved outside the grid or got onto a loop
            if beams_left_grid:
                for i in beams_left_grid[::-1]:  # starting at the end to not mess up indexing
                    self.beams.pop(i)
            # adding the new beams to the list
            self.beams += new_beams

    def count_energized(self):
        return len(self.energized)

    def new_start(self, start):
        self.energized.clear()
        self.beam_tracking.clear()
        self.beams.clear()
        self.beams.append(LaserBeam(*start))

    def print_energized(self):
        for r in range(0, self.grid.height):
            for c in range(0, self.grid.width) :
                if (r,c) in self.energized:
                    print("#", end="")
                else:
                    print(".", end="")
            
            print()
        
        print()


@timer_func
def day16(filepath, part2=False):
    with open(filepath) as fin:
        lines = [line.strip() for line in fin.readlines()]

    beam_map = BeamMap(lines)
    beam_map.project_beams()
    if not part2:
        beam_map.print_energized()
        return beam_map.count_energized()
    else:
        max_e = beam_map.count_energized()
        best_dir = None
        for d, y in [['r', -1], ['l', len(lines[0])]]:
            for x in range(len(lines)):
                if x == 0 and d == 'r':
                    continue
                beam_map.new_start(((x, y), d))
                beam_map.project_beams()
                e_c = beam_map.count_energized()
                if e_c > max_e:
                    best_dir = [d, y]
                    max_e = e_c
        for d, x in [['u', len(lines)], ['d', -1]]:
            for y in range(len(lines[0])):
                beam_map.new_start(((x, y), d))
                beam_map.project_beams()
                e_c = beam_map.count_energized()
                if e_c > max_e:
                    best_dir = [d, y]
                    max_e = e_c
        return (max_e, best_dir)


def main():
    # assert day16('input_sample.txt') == 46
    print(f"Part 1: {day16('input_puzzle.txt')}")

    # assert day16('input_sample.txt', True) == 51
    print(f"Part 2: {day16('input_puzzle.txt', True)}")


if __name__ == '__main__':
    main()