<details>
<summary>XSD contract: <code>TextFormatType</code> (SDMX 3.0)</summary>

```xml
	<xs:complexType name="TextFormatType">
		<xs:annotation>
			<xs:documentation>TextFormatType defines the information for describing a full range of text formats and may place restrictions on the values of the other attributes, referred to as "facets".</xs:documentation>
		</xs:annotation>
		<xs:sequence>
			<xs:element name="SentinelValue" type="SentinelValueType" minOccurs="0" maxOccurs="unbounded">
				<xs:annotation>
					<xs:documentation>SentinelValue defines a value that has a special meaning within the text format representation of a component.</xs:documentation>
				</xs:annotation>
			</xs:element>
		</xs:sequence>
		<xs:attribute name="textType" type="common:DataType" default="String">
			<xs:annotation>
				<xs:documentation>The textType attribute provides a description of the datatype. If it is not specified, any valid characters may be included in the text field (it corresponds to the xs:string datatype of W3C XML Schema) within the constraints of the facets.</xs:documentation>
			</xs:annotation>
		</xs:attribute>
		<xs:attribute name="isSequence" type="xs:boolean" use="optional">
			<xs:annotation>
				<xs:documentation>The isSequence attribute indicates whether the values are intended to be ordered, and it may work in combination with the interval, startValue, and endValue attributes or the timeInterval, startTime, and endTime, attributes. If this attribute holds a value of true, a start value or time and a numeric or time interval must supplied. If an end value is not given, then the sequence continues indefinitely.</xs:documentation>
			</xs:annotation>
		</xs:attribute>
		<xs:attribute name="interval" type="xs:decimal" use="optional">
			<xs:annotation>
				<xs:documentation>The interval attribute specifies the permitted interval (increment) in a sequence. In order for this to be used, the isSequence attribute must have a value of true.</xs:documentation>
			</xs:annotation>
		</xs:attribute>
		<xs:attribute name="startValue" type="xs:decimal" use="optional">
			<xs:annotation>
				<xs:documentation>The startValue attribute is used in conjunction with the isSequence and interval attributes (which must be set in order to use this attribute). This attribute is used for a numeric sequence, and indicates the starting  point of the sequence. This value is mandatory for a numeric sequence to be expressed.</xs:documentation>
			</xs:annotation>
		</xs:attribute>
		<xs:attribute name="endValue" type="xs:decimal" use="optional">
			<xs:annotation>
				<xs:documentation>The endValue attribute is used in conjunction with the isSequence and interval attributes (which must be set in order to use this attribute). This attribute is used for a numeric sequence, and indicates that ending point (if any) of the sequence.</xs:documentation>
			</xs:annotation>
		</xs:attribute>
		<xs:attribute name="timeInterval" type="xs:duration" use="optional">
			<xs:annotation>
				<xs:documentation>The timeInterval attribute indicates the permitted duration in a time sequence. In order for this to be used, the isSequence attribute must have a value of true.</xs:documentation>
			</xs:annotation>
		</xs:attribute>
		<xs:attribute name="startTime" type="common:StandardTimePeriodType" use="optional">
			<xs:annotation>
				<xs:documentation>The startTime attribute is used in conjunction with the isSequence and timeInterval attributes (which must be set in order to use this attribute). This attribute is used for a time sequence, and indicates the start time of the sequence. This value is mandatory for a time sequence to be expressed.</xs:documentation>
			</xs:annotation>
		</xs:attribute>
		<xs:attribute name="endTime" type="common:StandardTimePeriodType" use="optional">
			<xs:annotation>
				<xs:documentation>The endTime attribute is used in conjunction with the isSequence and timeInterval attributes (which must be set in order to use this attribute). This attribute is used for a time sequence, and indicates that ending point (if any) of the sequence.</xs:documentation>
			</xs:annotation>
		</xs:attribute>
		<xs:attribute name="minLength" type="xs:positiveInteger" use="optional">
			<xs:annotation>
				<xs:documentation>The minLength attribute specifies the minimum and length of the value in characters.</xs:documentation>
			</xs:annotation>
		</xs:attribute>
		<xs:attribute name="maxLength" type="xs:positiveInteger" use="optional">
			<xs:annotation>
				<xs:documentation>The maxLength attribute specifies the maximum length of the value in characters.</xs:documentation>
			</xs:annotation>
		</xs:attribute>
		<xs:attribute name="minValue" type="xs:decimal" use="optional">
			<xs:annotation>
				<xs:documentation>The minValue attribute is used for inclusive and exclusive ranges, indicating what the lower bound of the range is. If this is used with an inclusive range, a valid value will be greater than or equal to the value specified here. If the inclusive and exclusive data type is not specified (e.g. this facet is used with an integer data type), the value is assumed to be inclusive.</xs:documentation>
			</xs:annotation>
		</xs:attribute>
		<xs:attribute name="maxValue" type="xs:decimal" use="optional">
			<xs:annotation>
				<xs:documentation>The maxValue attribute is used for inclusive and exclusive ranges, indicating what the upper bound of the range is. If this is used with an inclusive range, a valid value will be less than or equal to the value specified here. If the inclusive and exclusive data type is not specified (e.g. this facet is used with an integer data type), the value is assumed to be inclusive.</xs:documentation>
			</xs:annotation>
		</xs:attribute>
		<xs:attribute name="decimals" type="xs:positiveInteger" use="optional">
			<xs:annotation>
				<xs:documentation>The decimals attribute indicates the number of characters allowed after the decimal separator.</xs:documentation>
			</xs:annotation>
		</xs:attribute>
		<xs:attribute name="pattern" type="xs:string" use="optional">
			<xs:annotation>
				<xs:documentation>The pattern attribute holds any regular expression permitted in the similar facet in W3C XML Schema.</xs:documentation>
			</xs:annotation>
		</xs:attribute>
		<xs:attribute name="isMultiLingual" type="xs:boolean" use="optional" default="true">
			<xs:annotation>
				<xs:documentation>The isMultiLingual attribute indicates for a text format of type "string", whether the value should allow for multiple values in different languages.</xs:documentation>
			</xs:annotation>
		</xs:attribute>
	</xs:complexType>
```

</details>
