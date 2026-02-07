import { ref } from 'vue';

export interface VariableMatch {
    name: string;
    startIndex: number;
    endIndex: number;
}

export function useVariableTooltip() {
    const tooltipVisible = ref(false);
    const tooltipPosition = ref({ x: 0, y: 0 });
    const hoveredVariable = ref('');
    const hoveredVariableValue = ref('');

    let hideTimeout: number | null = null;

    /**
     * Detect variables in text ({{variableName}})
     */
    const detectVariables = (text: string): VariableMatch[] => {
        const regex = /\{\{([^}]+)\}\}/g;
        const matches: VariableMatch[] = [];
        let match;

        while ((match = regex.exec(text)) !== null) {
            matches.push({
                name: match[1].trim(),
                startIndex: match.index,
                endIndex: match.index + match[0].length,
            });
        }

        return matches;
    };

    /**
     * More accurate way to get pixel position of a char index
     */
    const getXForIndex = (element: HTMLInputElement | HTMLTextAreaElement, index: number): number => {
        const style = window.getComputedStyle(element);
        const mirror = document.createElement('div');

        // Copy exact styling
        const props = [
            'font-family', 'font-size', 'font-weight', 'font-style', 'letter-spacing',
            'text-transform', 'word-spacing', 'text-indent', 'padding-left', 'border-left-width'
        ];

        props.forEach(p => mirror.style.setProperty(p, style.getPropertyValue(p)));
        mirror.style.position = 'absolute';
        mirror.style.visibility = 'hidden';
        mirror.style.whiteSpace = 'pre';

        // Measure up to the index
        mirror.textContent = element.value.substring(0, index);
        document.body.appendChild(mirror);
        const width = mirror.scrollWidth;
        document.body.removeChild(mirror);

        return width;
    };

    const handleInputMouseMove = (
        event: MouseEvent,
        inputElement: HTMLInputElement | HTMLTextAreaElement,
        variables: Record<string, string>
    ) => {
        const text = inputElement.value;
        const matches = detectVariables(text);

        if (matches.length === 0) {
            hideTooltip();
            return;
        }

        const { offsetX } = event;
        const rect = inputElement.getBoundingClientRect();

        // Iterate matches and find if mouse is over any
        for (const match of matches) {
            const startX = getXForIndex(inputElement, match.startIndex);
            const endX = getXForIndex(inputElement, match.endIndex);

            // Add a small buffer for easier hovering
            if (offsetX >= startX - 2 && offsetX <= endX + 2) {
                showTooltip(
                    match.name,
                    variables[match.name] || '',
                    rect.left + startX,
                    rect.top
                );
                return;
            }
        }

        hideTooltip();
    };

    const showTooltip = (variableName: string, value: string, x: number, y: number) => {
        if (hideTimeout) {
            clearTimeout(hideTimeout);
            hideTimeout = null;
        }

        hoveredVariable.value = variableName;
        hoveredVariableValue.value = value;
        tooltipPosition.value = { x, y };
        tooltipVisible.value = true;
    };

    const hideTooltip = () => {
        if (hideTimeout) return;
        hideTimeout = window.setTimeout(() => {
            tooltipVisible.value = false;
            hoveredVariable.value = '';
            hoveredVariableValue.value = '';
            hideTimeout = null;
        }, 100);
    };

    const closeTooltip = () => {
        if (hideTimeout) {
            clearTimeout(hideTimeout);
            hideTimeout = null;
        }
        tooltipVisible.value = false;
        hoveredVariable.value = '';
        hoveredVariableValue.value = '';
    };

    return {
        tooltipVisible,
        tooltipPosition,
        hoveredVariable,
        hoveredVariableValue,
        handleInputMouseMove,
        showTooltip,
        hideTooltip,
        closeTooltip,
    };
}
