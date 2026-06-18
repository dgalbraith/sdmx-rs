<details>
<summary>XSD contract: <code>AttributeRelationshipType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="AttributeRelationshipType">
		<xs:annotation>
			<xs:documentation>AttributeRelationshipType defines the structure for stating the relationship between an attribute and other data structure definition components.</xs:documentation>
		</xs:annotation>
		<xs:choice>
			<xs:element name="Dataflow" type="common:EmptyType">
				<xs:annotation>
					<xs:documentation>This means that the value of the attribute varies per dataflow. It is the data modeller's responsibility to design or use non-overlapping dataflows that do not have observations in common, otherwise the integrity of dataflow-specific attribute values is not assured by the model, e.g. when querying those data through its DSD.</xs:documentation>
				</xs:annotation>
			</xs:element>
			<xs:sequence>
				<xs:element name="Dimension" type="OptionalLocalDimensionReferenceType" maxOccurs="unbounded">
					<xs:annotation>
						<xs:documentation>This is used to reference dimensions in the data structure definition on which the value of this attribute depends. An attribute using this relationship can be either a group, series (or section), or observation level attribute. The attachment level of the attribute will be determined by the data format and which dimensions are referenced.</xs:documentation>
					</xs:annotation>
				</xs:element>
			</xs:sequence>
			<xs:element name="Group" type="common:IDType">
				<xs:annotation>
					<xs:documentation>This is used as a convenience to reference all of the dimension defined by the referenced group. The attribute will also be attached to this group.</xs:documentation>
				</xs:annotation>
			</xs:element>
			<xs:element name="Observation" type="common:EmptyType">
				<xs:annotation>
					<xs:documentation>This is used to specify that the value of the attribute is dependent upon the observed value. An attribute with this relationship will always be treated as an observation level attribute.</xs:documentation>
				</xs:annotation>
			</xs:element>
		</xs:choice>
	</xs:complexType>
```

</details>
